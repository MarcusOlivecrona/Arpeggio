#![allow(non_snake_case)]
use dioxus::prelude::*;
use dioxus_desktop::wry::application::accelerator::Accelerator;
use dioxus_desktop::DesktopService;
use dioxus_desktop::{tao::dpi::PhysicalPosition, use_window, LogicalSize, WindowBuilder};
use serde::Deserialize;
use std::process::Command;
use std::rc::Rc;
use std::str::FromStr;
use window_vibrancy::*;

const COLOR_COMMAND: &str = "#448895";
const COLOR_LAYER: &str = "#d77059";
const ESCAPE: &str = "escape";

fn main() {
    dioxus_desktop::launch_cfg(app, make_config());
}

type KeyLayer = Vec<KeyEntry>;

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum KeyLayerValue {
    Layer(KeyLayer),
    Command(String),
}

#[derive(Clone, Deserialize)]
struct KeyEntry {
    keychord: String,
    name: String,
    value: KeyLayerValue,
}

fn app(cx: Scope) -> Element {
    let window = use_window(cx);

    #[cfg(target_os = "macos")]
    let _ = apply_vibrancy(
        window.webview.window(),
        NSVisualEffectMaterial::HudWindow,
        Some(NSVisualEffectState::Active),
        Some(5.0),
    )
    .expect("Unsupported platform! 'apply_vibrancy' is only supported on macOS");

    let key_mapping: &UseState<Vec<KeyEntry>> = use_state(cx, load_key_mapping);
    let path: &UseRef<Vec<String>> = use_ref(cx, Vec::new);
    let mut active_layer = &*key_mapping.current();
    'outer: for keychord in path.read().iter() {
        if keychord == ESCAPE {
            std::process::exit(0);
        }
        for entry in active_layer {
            if keychord == &entry.keychord {
                match &entry.value {
                    KeyLayerValue::Layer(layer) => {
                        active_layer = layer;
                        continue 'outer;
                    }
                    KeyLayerValue::Command(command) => {
                        run_command(&command);
                        std::process::exit(0);
                    }
                }
            }
        }
        std::process::exit(0);
    }
    update_shortcuts(active_layer, path, window);
    cx.render(rsx! {
        div {
            width: "100%",
            height: "100%",
            padding: "30px",
            display: "flex",
            flex_wrap: "wrap",
            flex_direction: "column",

            for entry in active_layer {
                KeyItem {
                    keychord: entry.keychord.clone(),
                    name: entry.name.clone(),
                    is_layer: matches!(entry.value, KeyLayerValue::Layer(_)),
                }
            }
        }
    })
}

fn load_key_mapping() -> KeyLayer {
    let mut path = dirs::home_dir().expect("config dir not found");
    path.push(".config/arpeggio/keymap.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("failed to open file"))
        .expect("failed to parse json")
}

fn update_shortcuts(
    key_mapping: &KeyLayer,
    path: &UseRef<Vec<String>>,
    window: &Rc<DesktopService>,
) {
    window.remove_all_shortcuts();
    for entry in key_mapping {
        to_owned![path];
        to_owned![entry];
        window
            .create_shortcut(
                Accelerator::from_str(entry.keychord.clone().as_str()).unwrap(),
                move || path.with_mut(|path| path.push(entry.keychord.clone())),
            )
            .unwrap();
    }
    window
        .create_shortcut(Accelerator::from_str(ESCAPE).unwrap(), move || {
            std::process::exit(0)
        })
        .unwrap();
}

fn run_command(command: &str) -> std::process::Child {
    let parts: Vec<&str> = command.split_whitespace().collect();
    Command::new(parts[0])
        .args(&parts[1..])
        .spawn()
        .expect("failed to run command")
}

#[derive(PartialEq, Props)]
struct KeyProps {
    #[props(into)]
    keychord: String,

    #[props(into)]
    name: String,

    is_layer: bool,
}

fn KeyItem(cx: Scope<KeyProps>) -> Element {
    cx.render(rsx! {
        div {
            color: if cx.props.is_layer { COLOR_LAYER } else { COLOR_COMMAND },
            margin: "5px",
            "{cx.props.keychord} --> {cx.props.name}"
        }
    })
}

fn make_config() -> dioxus_desktop::Config {
    dioxus_desktop::Config::default()
        .with_window(make_window())
        .with_custom_head(
            r##"
    <style type="text/css">
        html, body {
            font-family: "FiraCode Nerd Font";
            font-size: 20px;
            height: 100%;
            margin: 0;
            overscroll-behavior-y: none;
            overscroll-behavior-x: none;
            overflow: hidden;
        }
        #main, #bodywrap {
            height: 100%;
            margin: 0;
            overscroll-behavior-x: none;
            overscroll-behavior-y: none;
        }
    </style>
    "##
            .to_owned(),
        )
}

fn make_window() -> WindowBuilder {
    WindowBuilder::new()
        .with_title("Arpeggio")
        .with_always_on_top(true)
        .with_visible(true)
        .with_transparent(true)
        .with_decorations(false)
        .with_resizable(false)
        .with_position(PhysicalPosition::new((3440 - 800) / 2, (1440 - 500) / 2))
        .with_inner_size(LogicalSize::new(800, 500))
}

#![allow(non_snake_case)]

use dioxus::dioxus_core::{ElementId, Mutation::*};
use dioxus::prelude::*;
use web::components::{Button, ThemedButton};

/// Test that Button renders correctly with minimal props
#[test]
fn test_button_renders_with_defaults() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                children: rsx!("Click me")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that Button renders with all props provided
#[test]
fn test_button_renders_with_all_props() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                class: "custom-class".to_string(),
                title: "Custom Title".to_string(),
                r#type: "submit".to_string(),
                disabled: true,
                children: rsx!("Submit")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that Button handles onclick handler
#[test]
fn test_button_with_onclick() {
    let mut dom = VirtualDom::new(|| {
        let mut clicked = use_signal(|| false);

        rsx! {
            Button {
                onclick: move |_| clicked.set(true),
                children: rsx!("Click me")
            }
            div { "{clicked}" }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that Button can be disabled
#[test]
fn test_button_disabled_state() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                disabled: true,
                children: rsx!("Disabled")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that ThemedButton renders with defaults
#[test]
fn test_themed_button_renders() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            ThemedButton {
                children: rsx!("Themed Button")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that ThemedButton applies extra classes
#[test]
fn test_themed_button_with_extra_classes() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            ThemedButton {
                class: "extra-class".to_string(),
                children: rsx!("Themed")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test that ThemedButton can be disabled
#[test]
fn test_themed_button_disabled() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            ThemedButton {
                disabled: true,
                children: rsx!("Disabled Themed")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test multiple buttons render side by side
#[test]
fn test_multiple_buttons_render() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            div {
                Button { children: rsx!("Button 1") }
                Button { children: rsx!("Button 2") }
                ThemedButton { children: rsx!("Themed Button") }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

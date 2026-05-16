#![allow(non_snake_case)]

use dioxus::prelude::*;
use web::components::ui::{Button, ButtonVariant};

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
}

fn primary_button() -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Primary,
            children: rsx!("Primary")
        }
    }
}

fn ghost_button() -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Ghost,
            children: rsx!("Ghost")
        }
    }
}

fn danger_button() -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Danger,
            children: rsx!("Danger")
        }
    }
}

fn chrome_button() -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Chrome,
            children: rsx!("Chrome")
        }
    }
}

#[test]
fn test_button_primary_variant() {
    let mut dom = VirtualDom::new(primary_button);
    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_button_ghost_variant() {
    let mut dom = VirtualDom::new(ghost_button);
    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_button_danger_variant() {
    let mut dom = VirtualDom::new(danger_button);
    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_button_chrome_variant() {
    let mut dom = VirtualDom::new(chrome_button);
    let _edits = dom.rebuild_to_vec();
}

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
}

#[test]
fn test_button_with_title() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                title: "Tooltip text".to_string(),
                children: rsx!("Hover me")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_button_submit_type() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                r#type: "submit".to_string(),
                children: rsx!("Submit")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_button_with_extra_class() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            Button {
                class: "w-full".to_string(),
                children: rsx!("Full width")
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_multiple_buttons_render() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            div {
                Button { variant: ButtonVariant::Primary, children: rsx!("Primary") }
                Button { variant: ButtonVariant::Ghost, children: rsx!("Ghost") }
                Button { variant: ButtonVariant::Danger, children: rsx!("Danger") }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
}

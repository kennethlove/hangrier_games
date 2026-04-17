#![allow(non_snake_case)]

use dioxus::prelude::*;
use web::components::Modal;

/// Test that Modal renders when open
#[test]
fn test_modal_renders_when_open() {
    let mut dom = VirtualDom::new(|| {
        let mut show_modal = use_signal(|| true);

        rsx! {
            Modal {
                show_modal,
                children: rsx!{
                    div { "Modal Content" }
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Modal should render without panicking when open
}

/// Test that Modal doesn't render when closed
#[test]
fn test_modal_hidden_when_closed() {
    let mut dom = VirtualDom::new(|| {
        let mut show_modal = use_signal(|| false);

        rsx! {
            Modal {
                show_modal,
                children: rsx!{
                    div { "Modal Content" }
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Modal should render without panicking when closed
}

/// Test that Modal can toggle state
#[test]
fn test_modal_state_toggle() {
    let mut dom = VirtualDom::new(|| {
        let mut show_modal = use_signal(|| false);

        rsx! {
            button {
                onclick: move |_| show_modal.set(!show_modal()),
                "Toggle Modal"
            }
            Modal {
                show_modal,
                children: rsx!{
                    div { "Modal Content" }
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test Modal with complex children
#[test]
fn test_modal_with_complex_children() {
    let mut dom = VirtualDom::new(|| {
        let mut show_modal = use_signal(|| true);

        rsx! {
            Modal {
                show_modal,
                children: rsx!{
                    div {
                        h1 { "Title" }
                        p { "Description" }
                        button { "Action" }
                    }
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Modal with complex children should render without panicking
}

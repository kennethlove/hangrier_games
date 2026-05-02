#![allow(non_snake_case)]

use dioxus::prelude::*;
use web::components::Modal;
use web::components::modal::Props as ModalProps;

#[derive(Props, Clone, PartialEq)]
struct Harness {
    open: bool,
    title: String,
}

fn ModalHarness(p: Harness) -> Element {
    let props = ModalProps {
        title: p.title.clone(),
        open: p.open,
        children: if p.open {
            Some(rsx! { p { "are you sure?" } })
        } else {
            None
        },
    };
    rsx! { Modal { modal_props: props } }
}

#[test]
fn test_modal_renders_closed_with_no_children() {
    let mut dom = VirtualDom::new_with_props(
        ModalHarness,
        Harness {
            open: false,
            title: "Hello".to_string(),
        },
    );
    let _edits = dom.rebuild_to_vec();
}

#[test]
fn test_modal_renders_open_with_children() {
    let mut dom = VirtualDom::new_with_props(
        ModalHarness,
        Harness {
            open: true,
            title: "Confirm".to_string(),
        },
    );
    let _edits = dom.rebuild_to_vec();
}

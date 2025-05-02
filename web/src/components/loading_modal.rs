use dioxus::prelude::*;
use crate::components::icons::mockingjay_arrow::MockingjayArrow;
use crate::components::modal::{Modal, Props as ModalProps};
use crate::LoadingState;

#[component]
pub fn LoadingModal() -> Element {
    let loading_signal = use_context::<Signal<LoadingState>>();

    let open = match *loading_signal.read() {
        LoadingState::Loading => true,
        _ => false,
    };

    let props = ModalProps {
        title: "Loading...".to_string(),
        open: open,
        children: Some(rsx! {
            div {
                class: "flex justify-center pb-4",
                MockingjayArrow {
                    class: r#"
                    size-16
                    motion-safe:animate-spin
                    motion-reduce:animate-pulse
                    theme1:fill-red-900
                    theme2:fill-green-800
                    theme3:fill-amber-600
                    theme3:size-24
                    "#
                }
            }
        })
    };

    rsx! {
        Modal { modal_props: props }
    }
}

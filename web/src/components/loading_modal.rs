use crate::components::modal::{Modal, Props as ModalProps};
use crate::LoadingState;
use dioxus::prelude::*;
use crate::components::icons::loading::LoadingIcon;

#[component]
pub fn LoadingModal() -> Element {
    let loading_signal = use_context::<Signal<LoadingState>>();

    let open = match *loading_signal.read() {
        LoadingState::Loading => true,
        _ => false,
    };

    let props = ModalProps {
        title: "Loading...".to_string(),
        open,
        children: Some(rsx! {
            div {
                class: "flex justify-center pb-4",
                LoadingIcon {}
            }
        })
    };

    rsx! {
        Modal { modal_props: props }
    }
}

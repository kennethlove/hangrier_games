use crate::LoadingState;
use crate::components::icons::loading::LoadingIcon;
use crate::components::modal::{Modal, Props as ModalProps};
use dioxus::prelude::*;

#[component]
pub fn LoadingModal() -> Element {
    let loading_signal = use_context::<Signal<LoadingState>>();

    let open = matches!(*loading_signal.read(), LoadingState::Loading);

    let props = ModalProps {
        title: "Loading...".to_string(),
        open,
        children: Some(rsx! {
            div {
                class: "flex justify-center pb-4",
                LoadingIcon {}
            }
        }),
    };

    rsx! {
        Modal { modal_props: props }
    }
}

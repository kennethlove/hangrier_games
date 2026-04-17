#![allow(non_snake_case)]

use dioxus::prelude::*;
use web::components::LoadingModal;

/// Test LoadingModal renders when loading is true
#[test]
fn test_loading_modal_visible() {
    let mut dom = VirtualDom::new(|| {
        let loading = use_signal(|| true);

        rsx! {
            LoadingModal { loading }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // LoadingModal should render without panicking when visible
}

/// Test LoadingModal hidden when loading is false
#[test]
fn test_loading_modal_hidden() {
    let mut dom = VirtualDom::new(|| {
        let loading = use_signal(|| false);

        rsx! {
            LoadingModal { loading }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // LoadingModal should render without panicking when hidden
}

/// Test LoadingModal state changes
#[test]
fn test_loading_modal_state_changes() {
    let mut dom = VirtualDom::new(|| {
        let mut loading = use_signal(|| false);

        rsx! {
            button {
                onclick: move |_| loading.set(true),
                "Start Loading"
            }
            LoadingModal { loading }
        }
    });

    let _edits = dom.rebuild_to_vec();

    // Trigger state change
    dom.mark_dirty(ScopeId::APP);
    let _edits2 = dom.render_immediate_to_vec();

    // Component should handle state change without panicking
}

/// Test LoadingModal in context of other components
#[test]
fn test_loading_modal_with_siblings() {
    let mut dom = VirtualDom::new(|| {
        let loading = use_signal(|| true);

        rsx! {
            div {
                h1 { "My App" }
                p { "Some content" }
                LoadingModal { loading }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // LoadingModal should coexist with other components
}

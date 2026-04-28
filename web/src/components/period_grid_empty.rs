//! PeriodGridEmpty — empty/error state for the timeline period grid.

use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub enum EmptyKind {
    NotStarted,
    LoadFailed,
    NotFound,
}

#[derive(Props, PartialEq, Clone)]
pub struct PeriodGridEmptyProps {
    pub kind: EmptyKind,
    #[props(default)]
    pub on_retry: Option<EventHandler<()>>,
}

#[component]
pub fn PeriodGridEmpty(props: PeriodGridEmptyProps) -> Element {
    let copy = match props.kind {
        EmptyKind::NotStarted => "This game hasn't started yet. Click Begin to start.",
        EmptyKind::LoadFailed => "Couldn't load the timeline.",
        EmptyKind::NotFound => "Game not found.",
    };
    rsx! {
        div { class: "rounded-lg border border-dashed p-8 text-center text-sm",
            p { "{copy}" }
            if matches!(props.kind, EmptyKind::LoadFailed)
                && let Some(retry) = props.on_retry
            {
                button {
                    class: "mt-4 rounded bg-amber-500 px-3 py-1 text-amber-50 hover:bg-amber-600",
                    onclick: move |_| retry.call(()),
                    "Retry"
                }
            }
        }
    }
}

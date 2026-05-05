use crate::components::ui::{Button, ButtonVariant, TopBar};
use crate::routes::Routes;
use crate::storage::{AppState, use_persistent};
use crate::theme::Theme;
use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);
    let mut theme_signal: Signal<Theme> = use_context();
    use_context_provider(|| Signal::new(crate::components::timeline::PeriodFilters::default()));

    let toggle_theme = move |_| {
        let next = theme_signal.read().toggle();
        theme_signal.set(next);
        let mut state = storage.get();
        state.set_theme(next);
        storage.set(state);
    };

    let signed_in = storage.get().username.is_some();
    let label = match *theme_signal.read() {
        Theme::Dark => "☀ Light",
        Theme::Light => "☾ Dark",
    };

    rsx! {
        TopBar { brand: "HANGRIER GAMES".to_string(),
            nav {
                aria_label: "Main navigation",
                class: "flex items-center gap-6 font-text font-bold text-[11px] uppercase tracking-[0.16em] text-text-muted",
                Link { class: "hover:text-text", to: Routes::Home {}, "Home" }
                if signed_in {
                    Link { class: "hover:text-text", to: Routes::GamesList {}, "Games" }
                }
                Link { class: "hover:text-text", to: Routes::AccountsPage {}, "Account" }
            }
            div { class: "ml-auto",
                Button { variant: ButtonVariant::Ghost, onclick: toggle_theme, "{label}" }
            }
        }
        main {
            class: "mx-auto max-w-3xl sm:max-w-3/4 py-6 px-2",
            Outlet::<Routes> {}
        }
    }
}

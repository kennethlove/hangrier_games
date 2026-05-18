use maud::{html, Markup, PreEscaped, DOCTYPE};

pub mod pages;

const SPRITES_UI: &str = include_str!("sprites_ui.svg");
const SPRITES_NARRATIVE: &str = include_str!("sprites_narrative.svg");

pub fn base_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Hangrier Games" }
                link rel="stylesheet" href="/assets/main.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                script src="https://unpkg.com/htmx-ext-sse@2.2.3" {}
            }
            body class="bg-gray-950 text-gray-100 min-h-screen" {
                nav class="bg-gray-900 border-b border-gray-800 px-4 py-3" {
                    div class="max-w-6xl mx-auto flex items-center justify-between" {
                        a href="/" class="text-lg font-bold text-amber-400" { "Hangrier Games" }
                        div class="flex items-center gap-4" {
                            a href="/games" class="text-sm text-gray-300 hover:text-white" { "Games" }
                            a href="/login" class="text-sm text-gray-300 hover:text-white" { "Login" }
                        }
                    }
                }
                main class="max-w-6xl mx-auto px-4 py-6" {
                    (content)
                }
                div class="hidden" { (PreEscaped(SPRITES_UI)) (PreEscaped(SPRITES_NARRATIVE)) }
            }
        }
    }
}

pub fn icon(name: &str) -> Markup {
    html! {
        svg class="inline w-4 h-4" {
            use href=(format!("#icon_ui_{}", name)) {}
        }
    }
}

pub fn narrative_icon(name: &str) -> Markup {
    html! {
        svg class="inline w-4 h-4" {
            use href=(format!("#icon_narrative_{}", name)) {}
        }
    }
}

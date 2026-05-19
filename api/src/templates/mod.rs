use maud::{DOCTYPE, Markup, PreEscaped, html};

pub mod auth;
pub mod game_detail;
pub mod pages;
pub mod timeline;

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
                // SVG sprites served as static files — browser caches after first load
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-ui.svg"/></svg>"#))
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-narrative.svg"/></svg>"#))
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

use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        h1 {
            class: r#"
            place-content-center
            cinzel-font
            text-6xl
            text-center
            font-bold
            bg-clip-text
            bg-radial
            theme1:text-transparent
            theme1:from-amber-300
            theme1:to-red-600
            drop-shadow-sm
            "#,
            "May the odds be ever in your favor!"
        }
    }
}


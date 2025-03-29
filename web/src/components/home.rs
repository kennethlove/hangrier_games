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

            theme1:bg-radial
            theme1:text-transparent
            theme1:from-amber-300
            theme1:to-red-600

            theme2:text-transparent
            theme2:bg-linear-to-b
            theme2:from-teal-600
            theme2:to-green-400

            theme3:text-slate-300
            "#,
            "May the odds be ever in your favor!"
        }
    }
}


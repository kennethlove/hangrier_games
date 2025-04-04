use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: r#"
            place-content-center
            min-h-full
            "#,

            h1 {
                class: r#"
                text-6xl
                text-center
                font-bold
                bg-clip-text
                drop-shadow-md
                sm:w-1/2
                mx-auto

                theme1:font-[Cinzel]
                theme1:bg-radial
                theme1:text-transparent
                theme1:from-amber-300
                theme1:to-red-600

                theme2:font-[Forum]
                theme2:text-transparent
                theme2:bg-linear-to-b
                theme2:to-teal-500
                theme2:from-green-400
                theme2:pb-2

                theme3:text-slate-700
                "#,
                "May the odds be ever in your favor!"
            }

            img {
                class: "mx-auto invisible theme3:hidden theme2:hidden theme1:visible",
                src: asset!("/assets/images/red.png"),
                alt: "Hunger Games"
            }
            img {
                class: "mx-auto invisible theme2:visible theme1:hidden theme3:hidden theme2:hidden",
                src: asset!("/assets/images/green.png"),
                alt: "Hunger Games"
            }
            img {
                class: "mx-auto invisible theme3:visible theme1:hidden theme2:hidden",
                src: asset!("/assets/images/blue.png"),
                alt: "Hunger Games"
            }
        }
    }
}

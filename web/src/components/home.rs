use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: r#"
            flex
            justify-center
            items-center
            min-h-[80vh]
            w-full
            place-items-center
            "#,
            h1 {
                class: r#"
                text-6xl
                text-center
                font-bold
                bg-clip-text
                drop-shadow-md/25

                place-self-center

                sm:w-2/3

                theme1:font-[Cinzel]
                theme1:bg-radial
                theme1:text-transparent
                theme1:from-amber-300
                theme1:to-red-500

                theme2:font-[Forum]
                theme2:text-transparent
                theme2:bg-linear-to-b
                theme2:to-teal-500
                theme2:from-green-400
                theme2:pb-2

                theme3:font-[Orbitron]
                theme3:text-transparent
                theme3:bg-gold-rich
                theme3:leading-[1.24]
                "#,

                "May the odds be ever in your flavor!"
            }
        }
    }
}

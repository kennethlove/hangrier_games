use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            class: "flex justify-center items-center min-h-[80vh] w-full place-items-center",
            h1 {
                class: "text-6xl text-center place-self-center lg:w-2/3 font-display uppercase tracking-wide text-text drop-shadow-md/25",
                "May the odds be ever in your flavor!"
            }
        }
    }
}

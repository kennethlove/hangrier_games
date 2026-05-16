use crate::routes::Routes;
use dioxus::prelude::*;

#[component]
pub fn NavLink(to: Routes, children: Element) -> Element {
    let route = use_route::<Routes>();
    let is_active = route == to;

    let class = if is_active {
        "hover:text-text border-b-2 border-gold pb-0.5"
    } else {
        "hover:text-text"
    };

    rsx! {
        Link { class, to, {children} }
    }
}

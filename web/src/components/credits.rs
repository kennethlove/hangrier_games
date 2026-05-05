use dioxus::prelude::*;

const LINK_CLASS: &str = "text-primary hover:underline";
const H2_CLASS: &str = "font-display text-3xl tracking-wide text-text mt-4";

#[component]
pub fn Credits() -> Element {
    rsx! {
        div {
            class: "mt-4 py-4 px-4 text-center bg-surface text-text rounded-card border border-border",

            h2 { class: H2_CLASS, "Credits" }

            p {
                "This game is a tribute to the work of Suzanne Collins and her series of books. ",
                "All copyrights and trademarks are the property of their respective owners.",
            }
            p { "Special thanks to everyone at work who likes to play." }

            h2 { class: H2_CLASS, "Tools" }
            p { "This game was created with:" }
            ul {
                li { a { class: LINK_CLASS, href: "https://www.rust-lang.org/", "Rust" } }
                li { a { class: LINK_CLASS, href: "https://github.com/tokio-rs/axum", "Axum" } }
                li { a { class: LINK_CLASS, href: "https://dioxuslabs.com/", "Dioxus" } }
                li { a { class: LINK_CLASS, href: "https://tailwindcss.com/", "Tailwind" } }
            }

            h2 { class: H2_CLASS, "Resources" }
            dl {
                class: "grid gap-4 grid-cols-1 sm:grid-cols-2",
                dt { "Mockingjay icons" }
                dd { a { class: LINK_CLASS, href: "https://www.vecteezy.com/members/inna-marchenko601727", "Inna Marchenko" } }
                dt { "Utility icons" }
                dd { a { class: LINK_CLASS, href: "https://www.heroicons.com", "heroicons" } }
                dt { "Background patterns" }
                dd { a { class: LINK_CLASS, href: "https://www.heropatterns.com", "Hero Patterns" } }
                dt { "Google fonts" }
                dd {
                    ul {
                        li { a { class: LINK_CLASS, href: "https://fonts.google.com/specimen/Bebas+Neue", "Bebas Neue" } }
                        li { a { class: LINK_CLASS, href: "https://fonts.google.com/specimen/Source+Sans+3", "Source Sans 3" } }
                        li { a { class: LINK_CLASS, href: "https://fonts.google.com/specimen/IBM+Plex+Mono", "IBM Plex Mono" } }
                    }
                }
                dt { "Game icons" }
                dd {
                    table {
                        class: "w-full border border-border mb-4",
                        thead {
                            tr {
                                th { class: "border border-border", "Author" }
                                th { class: "border border-border", "Icons" }
                            }
                        }
                        tbody {
                            tr {
                                td { class: "border border-border", "Lorc" }
                                td { class: "border border-border",
                                    ul {
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/broken-bone.html", "Broken Bone" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/desert-skull.html", "Desert Skull" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/drowning.html", "Drowning" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/fire-silhouette.html", "Fire Silhouette" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/fishing-net.html", "Fishing Net" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/fist.html", "Fist" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/fizzing-flask.html", "Fizzing Flask" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/harpoon-trident.html", "Harpoon Trident" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/heat-haze.html", "Heat Haze" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/high-shot.html", "High Shot" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/hypodermic-test.html", "Hypodermic Test" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/lightning-electron.html", "Lightning Electron" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/plain-dagger.html", "Plain Dagger" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/pointy-sword.html", "Pointy Sword" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/poison-bottle.html", "Poison Bottle" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/powder.html", "Powder" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/spear-hook.html", "Spear Hook" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/spiked-mace.html", "Spiked Mace" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/spray.html", "Spray" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/sticking-plaster.html", "Sticking Plaster" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/top-paw.html", "Top Paw" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/vomiting.html", "Vomiting" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/lorc/wood-axe.html", "Wood Axe" } }
                                    }
                                }
                            }
                            tr {
                                td { class: "border border-border", "Delapouite" }
                                td { class: "border border-border",
                                    ul {
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/delapouite/chips-bag.html", "Chips Bag" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/delapouite/falling-rocks.html", "Falling Rocks" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/delapouite/frozen-body.html", "Frozen Body" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/delapouite/half-dead.html", "Half Dead" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/delapouite/stomach.html", "Stomach" } }
                                    }
                                }
                            }
                            tr {
                                td { class: "border border-border", "sbed" }
                                td { class: "border border-border",
                                    ul {
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/sbed/death-skull.html", "Death Skull" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/sbed/shield.html", "Shield" } }
                                    }
                                }
                            }
                            tr {
                                td { class: "border border-border", "Skoll" }
                                td { class: "border border-border",
                                    ul {
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/skoll/hearts.html", "Hearts" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/skoll/spinning-top.html", "Spinning Top" } }
                                        li { a { class: LINK_CLASS, href: "https://game-icons.net/1x1/skoll/switchblade.html", "Switchblade" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

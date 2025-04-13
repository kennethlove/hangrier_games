use dioxus::prelude::*;

#[component]
pub fn Credits() -> Element {
    rsx! {
        div {
            class: r#"
            mt-4
            py-2
            px-4
            text-center
            theme1:text-stone-200
            theme1:bg-stone-800/50
            "#,

            h2 {
                class: r#"
                text-3xl
                theme1:font-[Cinzel]
                "#,
                "Credits"
            }

            p {
                "This game is a tribute to the work of Suzanne Collins and her series of books. ",
                "All copyrights and trademarks are the property of their respective owners.",
            }
            p { "Special thanks to everyone at work who likes to play." }

            h2 {
                class: r#"
                mt-4
                text-3xl
                theme1:font-[Cinzel]
                "#,
                "Tools"
            }
            p { "This game was created with:" }
            ul {
                li {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://www.rust-lang.org/",
                        "Rust"
                    }
                }
                li {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://github.com/tokio-rs/axum",
                        "Axum"
                    }
                }
                li {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://dioxuslabs.com/",
                        "Dioxus"
                    }
                }
                li {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://tailwindcss.com/",
                        "Tailwind"
                    }
                }
            }

            h2 {
                class: r#"
                text-3xl
                mt-4
                theme1:font-[Cinzel]
                "#,
                "Resources"
            }
            dl {
                class: "grid gap-4 grid-cols-1 sm:grid-cols-2",
                dt { "Mockingjay icons" }
                dd {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://www.vecteezy.com/members/inna-marchenko601727",
                        "Inna Marchenko"
                    }
                }
                dt { "Utility icons" }
                dd {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://www.heroicons.com",
                        "heroicons"
                    }
                }
                dt {
                    "Background patterns"
                }
                dd {
                    a {
                        class: "theme1:text-amber-300",
                        href: "https://www.heropatterns.com",
                        "Hero Patterns"
                    }
                }
                dt { "Google fonts" }
                dd {
                    ul {
                        li {
                            a {
                                class: "theme1:text-amber-300",
                                href: "https://fonts.google.com/specimen/Cinzel",
                                "Cinzel"
                            }
                        }
                        li {
                            a {
                                class: "theme1:text-amber-300",
                                href: "https://fonts.google.com/specimen/Forum",
                                "Forum"
                            }
                        }
                        li {
                            a {
                                class: "theme1:text-amber-300",
                                href: "https://fonts.google.com/specimen/Orbitron",
                                "Orbitron"
                            }
                        }
                        li {
                            a {
                                class: "theme1:text-amber-300",
                                href: "https://fonts.google.com/specimen/Work+Sans",
                                "Work Sans"
                            }
                        }
                    }
                }
                dt { "Game icons" }
                dd {
                    table {
                        class: "w-full border border-gray-400 mb-4",
                        thead {
                            tr {
                                th {
                                    class: "border border-gray-300",
                                    "Author"
                                }
                                th {
                                    class: "border border-gray-300",
                                    "Icons"
                                }
                            }
                        }
                        tbody {
                            tr {
                                td {
                                    class: "border border-gray-300",
                                    "Lorc"
                                }
                                td {
                                    class: "border border-gray-300",
                                    ul {
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/broken-bone.html",
                                                "Broken Bone"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/desert-skull.html",
                                                "Desert Skull"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/drowning.html",
                                                "Drowning"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/fire-silhouette.html",
                                                "Fire Silhouette"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/fishing-net.html",
                                                "Fishing Net"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/fist.html",
                                                "Fist"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/fizzing-flask.html",
                                                "Fizzing Flask"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/harpoon-trident.html",
                                                "Harpoon Trident"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/heat-haze.html",
                                                "Heat Haze"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/high-shot.html",
                                                "High Shot"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/hypodermic-test.html",
                                                "Hypodermic Test"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/lightning-electron.html",
                                                "Lightning Electron"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/plain-dagger.html",
                                                "Plain Dagger"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/pointy-sword.html",
                                                "Pointy Sword"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/poison-bottle.html",
                                                "Poison Bottle"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/powder.html",
                                                "Powder"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/spear-hook.html",
                                                "Spear Hook"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/spiked-mace.html",
                                                "Spiked Mace"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/spray.html",
                                                "Spray"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/sticking-plaster.html",
                                                "Sticking Plaster"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/top-paw.html",
                                                "Top Paw"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/lorc/vomiting.html",
                                                "Vomiting"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border border-gray-300",
                                    "Delapouite"
                                }
                                td {
                                    class: "border border-gray-300",
                                    ul {
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/delapouite/chips-bag.html",
                                                "Chips Bag"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/delapouite/falling-rocks.html",
                                                "Falling Rocks"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/delapouite/frozen-body.html",
                                                "Frozen Body"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/delapouite/half-dead.html",
                                                "Half Dead"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/delapouite/stomach.html",
                                                "Stomach"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border border-gray-300",
                                    "sbed"
                                }
                                td {
                                    class: "border border-gray-300",
                                    ul {
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/sbed/death-skull.html", "Death Skull"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/sbed/health-normal.html",
                                                "Health Normal"
                                            }
                                        }
                                        li {
                                            a {
                                                class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/sbed/shield.html",
                                                "Shield"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border border-gray-300",
                                    "Skoll"
                                }
                                td {
                                    class: "border border-gray-300",
                                    ul {
                                        li {
                                            a {
                                        class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/skoll/spinning-top.html",
                                                "Spinning Top"
                                            }
                                        }
                                        li {
                                            a {
                                        class: "theme1:text-amber-300",
                                                href: "https://game-icons.net/1x1/skoll/switchblade.html",
                                                "Switchblade"
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
    }
}

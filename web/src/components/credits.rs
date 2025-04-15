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
            theme2:text-green-200
            theme3:bg-linear-to-b
            theme3:from-stone-50/20
            theme3:to-stone-50
            "#,

            h2 {
                class: r#"
                theme1:text-3xl
                theme1:font-[Cinzel]
                theme2:text-4xl
                theme2:font-[Forum]
                theme3:text-3xl
                theme3:font-[Orbitron]
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
                theme1:text-3xl
                theme1:font-[Cinzel]
                theme2:text-4xl
                theme2:font-[Forum]
                theme3:text-3xl
                theme3:font-[Orbitron]
                "#,
                "Tools"
            }
            p { "This game was created with:" }
            ul {
                li {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://www.rust-lang.org/",
                        "Rust"
                    }
                }
                li {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://github.com/tokio-rs/axum",
                        "Axum"
                    }
                }
                li {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://dioxuslabs.com/",
                        "Dioxus"
                    }
                }
                li {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://tailwindcss.com/",
                        "Tailwind"
                    }
                }
            }

            h2 {
                class: r#"
                mt-4
                theme1:text-3xl
                theme1:font-[Cinzel]
                theme2:text-4xl
                theme2:font-[Forum]
                theme3:text-3xl
                theme3:font-[Orbitron]
                "#,
                "Resources"
            }
            dl {
                class: "grid gap-4 grid-cols-1 sm:grid-cols-2",
                dt { "Mockingjay icons" }
                dd {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://www.vecteezy.com/members/inna-marchenko601727",
                        "Inna Marchenko"
                    }
                }
                dt { "Utility icons" }
                dd {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://www.heroicons.com",
                        "heroicons"
                    }
                }
                dt {
                    "Background patterns"
                }
                dd {
                    a {
                        class: r#"
                        theme1:text-amber-300
                        theme2:underline
                        theme2:decoration-wavy
                        theme3:text-yellow-600
                        "#,
                        href: "https://www.heropatterns.com",
                        "Hero Patterns"
                    }
                }
                dt { "Google fonts" }
                dd {
                    ul {
                        li {
                            a {
                                class: r#"
                                theme1:text-amber-300
                                theme2:underline
                                theme2:decoration-wavy
                                theme3:text-yellow-600
                                "#,
                                href: "https://fonts.google.com/specimen/Cinzel",
                                "Cinzel"
                            }
                        }
                        li {
                            a {
                                class: r#"
                                theme1:text-amber-300
                                theme2:underline
                                theme2:decoration-wavy
                                theme3:text-yellow-600
                                "#,
                                href: "https://fonts.google.com/specimen/Forum",
                                "Forum"
                            }
                        }
                        li {
                            a {
                                class: r#"
                                theme1:text-amber-300
                                theme2:underline
                                theme2:decoration-wavy
                                theme3:text-yellow-600
                                "#,
                                href: "https://fonts.google.com/specimen/Orbitron",
                                "Orbitron"
                            }
                        }
                        li {
                            a {
                                class: r#"
                                theme1:text-amber-300
                                theme2:underline
                                theme2:decoration-wavy
                                theme3:text-yellow-600
                                "#,
                                href: "https://fonts.google.com/specimen/Work+Sans",
                                "Work Sans"
                            }
                        }
                    }
                }
                dt { "Game icons" }
                dd {
                    table {
                        class: "w-full border mb-4",
                        thead {
                            tr {
                                th {
                                    class: "border",
                                    "Author"
                                }
                                th {
                                    class: "border",
                                    "Icons"
                                }
                            }
                        }
                        tbody {
                            tr {
                                td {
                                    class: "border",
                                    "Lorc"
                                }
                                td {
                                    class: "border",
                                    ul {
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/broken-bone.html",
                                                "Broken Bone"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/desert-skull.html",
                                                "Desert Skull"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/drowning.html",
                                                "Drowning"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/fire-silhouette.html",
                                                "Fire Silhouette"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/fishing-net.html",
                                                "Fishing Net"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/fist.html",
                                                "Fist"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/fizzing-flask.html",
                                                "Fizzing Flask"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/harpoon-trident.html",
                                                "Harpoon Trident"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/heat-haze.html",
                                                "Heat Haze"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/high-shot.html",
                                                "High Shot"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/hypodermic-test.html",
                                                "Hypodermic Test"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/lightning-electron.html",
                                                "Lightning Electron"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/plain-dagger.html",
                                                "Plain Dagger"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/pointy-sword.html",
                                                "Pointy Sword"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/poison-bottle.html",
                                                "Poison Bottle"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/powder.html",
                                                "Powder"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/spear-hook.html",
                                                "Spear Hook"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/spiked-mace.html",
                                                "Spiked Mace"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/spray.html",
                                                "Spray"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/sticking-plaster.html",
                                                "Sticking Plaster"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/top-paw.html",
                                                "Top Paw"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/lorc/vomiting.html",
                                                "Vomiting"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border",
                                    "Delapouite"
                                }
                                td {
                                    class: "border",
                                    ul {
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/delapouite/chips-bag.html",
                                                "Chips Bag"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/delapouite/falling-rocks.html",
                                                "Falling Rocks"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/delapouite/frozen-body.html",
                                                "Frozen Body"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/delapouite/half-dead.html",
                                                "Half Dead"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/delapouite/stomach.html",
                                                "Stomach"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border",
                                    "sbed"
                                }
                                td {
                                    class: "border",
                                    ul {
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/sbed/death-skull.html", "Death Skull"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/sbed/shield.html",
                                                "Shield"
                                            }
                                        }
                                    }
                                }
                            }
                            tr {
                                td {
                                    class: "border",
                                    "Skoll"
                                }
                                td {
                                    class: "border",
                                    ul {
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/skoll/hearts.html",
                                                "Hearts"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
                                                href: "https://game-icons.net/1x1/skoll/spinning-top.html",
                                                "Spinning Top"
                                            }
                                        }
                                        li {
                                            a {
                                                class: r#"
                                                theme1:text-amber-300
                                                theme2:underline
                                                theme2:decoration-wavy
                                                theme3:text-yellow-600
                                                "#,
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

use dioxus::prelude::*;
use crate::components::{Button, Input, ThemedButton};
use crate::routes::Routes;

#[component]
pub fn Accounts() -> Element {
    rsx! {
        div {
            id: "accounts",
            Outlet::<Routes> {}
        }
    }
}

#[component]
pub fn AccountsPage() -> Element {
    rsx! {
        div {
            class: r#"
            grid
            grid-cols-1
            sm:grid-cols-2
            my-4
            "#,
            div {
                class: "pb-6 sm:px-6",
                div {
                    class: "flex flex-col gap-4",
                    h2 {
                        class: r#"
                        text-xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-500
                        theme2:font-[Playfair_Display]
                        theme2:text-green-300
                        theme2:font-bold
                        theme2:tracking-wide
                        "#,

                        "Login"
                    }
                    form {
                        class: "flex flex-col gap-2",
                        id: "login-form",
                        method: "POST",
                        Input {
                            r#type: "email",
                            name: "email",
                            placeholder: "Email"
                        }
                        Input {
                            r#type: "password",
                            name: "password",
                            placeholder: "Password"
                        }
                        div {
                            class: "clear",
                            ThemedButton {
                                r#type: "submit",
                                "Login"
                            }
                        }
                    }
                }
            }
            div {
                class: "pt-4 sm:px-4 sm:pt-0",
                div {
                    class: "flex flex-col gap-4",
                    h2 {
                        class: r#"
                        text-xl
                        theme1:font-[Cinzel]
                        theme1:text-amber-500
                        theme2:font-[Playfair_Display]
                        theme2:text-green-300
                        theme2:font-bold
                        theme2:tracking-wide
                        theme3:font-[Orbitron]
                        theme3:text-stone-700
                        "#,

                        "Register"
                    }
                    form {
                        class: "flex flex-col gap-2",
                        id: "register-form",
                        method: "POST",
                        Input {
                            r#type: "email",
                            name: "email",
                            placeholder: "Email"
                        }
                        Input {
                            r#type: "password",
                            name: "password",
                            placeholder: "Password"
                        }
                        Input {
                            r#type: "password",
                            name: "password2",
                            placeholder: "Password again"
                        }
                        div {
                            ThemedButton {
                                r#type: "submit",
                                "Register"
                            }
                        }
                    }
                }
            }
        }
        p {
            class: "text-sm text-center theme1:text-amber-200 theme2:text-green-200 theme3:text-stone-200",
            "Your email and password are stored in a secure database. We do not share your data with third parties."
        }
    }
}

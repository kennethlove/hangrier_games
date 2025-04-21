use dioxus::prelude::*;
use crate::components::Button;
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

#[derive(Clone, Debug, PartialEq, Props)]
struct InputProperties {
    r#type: String,
    name: String,
    placeholder: Option<String>,
    id: Option<String>,
    value: Option<String>,
}

#[component]
fn Input(props: InputProperties) -> Element {
    rsx! {
        input {
            r#type: "{props.clone().r#type}",
            name: "{props.clone().name}",
            id: "{props.clone().id.unwrap_or_default()}",
            value: "{props.clone().value.unwrap_or_default()}",
            placeholder: "{props.clone().placeholder.unwrap_or_default()}",

            class: r#"
            block
            border
            w-half
            px-2
            py-1
            transition

            theme1:border-amber-600
            theme1:text-amber-200
            theme1:placeholder-amber-200/50
            theme1:bg-stone-800/65
            theme1:hover:bg-stone-800/75
            theme1:focus:bg-stone-800/75

            theme2:border-green-400
            theme2:text-green-200
            theme2:placeholder-green-200/50

            theme3:bg-stone-50/50
            theme3:border-yellow-600
            theme3:placeholder-stone-500
            theme3:text-stone-800
            "#,
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
            divide-y
            sm:grid-cols-2
            sm:divide-x
            sm:divide-y-0
            my-4
            place-items-center
            theme1:divide-stone-800/50
            theme2:divide-green-800
            theme3:divide-stone-700
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
                            Button {
                                extra_classes: Some(r#"
                                theme1:bg-radial
                                theme1:from-amber-300
                                theme1:to-red-500
                                theme1:border-red-500
                                theme1:text-red-900
                                theme1:hover:text-stone-200
                                theme1:hover:from-amber-500
                                theme1:hover:to-red-700

                                theme2:text-green-800
                                theme2:bg-linear-to-b
                                theme2:from-green-400
                                theme2:to-teal-500
                                theme2:border-none
                                theme2:hover:text-green-200
                                theme2:hover:from-green-500
                                theme2:hover:to-teal-600

                                theme3:border-none
                                theme3:bg-gold-rich
                                theme3:hover:bg-gold-rich-reverse
                                theme3:text-stone-700
                                theme3:hover:text-stone-50
                                "#.into()),
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
                            Button {
                                extra_classes: Some(r#"
                                theme1:bg-radial
                                theme1:from-amber-300
                                theme1:to-red-500
                                theme1:border-red-500
                                theme1:text-red-900
                                theme1:hover:text-stone-200
                                theme1:hover:from-amber-500
                                theme1:hover:to-red-700

                                theme2:text-green-800
                                theme2:bg-linear-to-b
                                theme2:from-green-400
                                theme2:to-teal-500
                                theme2:border-none
                                theme2:hover:text-green-200
                                theme2:hover:from-green-500
                                theme2:hover:to-teal-600

                                theme3:border-none
                                theme3:bg-gold-rich
                                theme3:hover:bg-gold-rich-reverse
                                theme3:text-stone-700
                                theme3:hover:text-stone-50
                                "#.into()),
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

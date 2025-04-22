use std::ops::Deref;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult};
use game::games::Game;
use shared::{AuthenticatedUser, RegistrationUser};
use crate::API_HOST;
use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::{Button, Input, ThemedButton};
use crate::routes::Routes;

async fn register_user(user: RegistrationUser) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = serde_json::json!({
        "email": user.email,
        "pass": user.password,
    });

    let response = client.post(format!("{}/api/users", API_HOST))
        .json(&json_body)
        .send().await;

    match response {
        Ok(response) => {
            match response.json::<AuthenticatedUser>().await {
                Ok(user) => {
                    MutationResult::Ok(MutationValue::NewUser(user))
                }
                Err(_) => {
                    MutationResult::Err(MutationError::UnableToCreateUser)
                }
            }
        }
        Err(e) => {
            dioxus_logger::tracing::error!("error creating user: {:?}", e);
            MutationResult::Err(MutationError::UnableToCreateUser)
        }
    }
}

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
            LoginForm {}
            RegisterForm {}
        }
        p {
            class: "text-sm text-center theme1:text-amber-200 theme2:text-green-200 theme3:text-stone-200",
            "Your email and password are stored in a secure database. We do not share your data with third parties."
        }
    }
}

#[component]
fn LoginForm() -> Element {
    rsx! {
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
    }
}

#[component]
fn RegisterForm() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let mut email_signal = use_signal(String::default);
    let mut password_signal = use_signal(String::default);
    let mut password2_signal = use_signal(String::default);
    let mut disabled_signal = use_signal(|| false);
    let mut email_error_signal = use_signal(String::default);
    let mut password_error_signal = use_signal(String::default);

    let mutate = use_mutation(register_user);

    rsx! {
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
                    onsubmit: move |_| {
                        email_error_signal.set(String::default());
                        password_error_signal.set(String::default());

                        let email = email_signal.read().clone();
                        if email.is_empty() {
                            email_error_signal.set("Email is required".to_string());
                        }

                        let password = password_signal.read().clone();
                        let password2 = password2_signal.read().clone();

                        if password != password2 {
                            password_error_signal.set("Passwords do not match".to_string());
                        }

                        if email_error_signal.peek().is_empty() && password_error_signal.peek().is_empty() {
                            disabled_signal.set(true);

                            spawn(async move {
                                let user = RegistrationUser {
                                    email: email.clone(),
                                    password: password.clone(),
                                };
                                mutate.manual_mutate(user).await;
                                if mutate.result().is_ok() {
                                    match mutate.result().deref() {
                                        MutationResult::Ok(MutationValue::NewUser(_user)) => {
                                            client.invalidate_queries(&[QueryKey::User]);
                                            disabled_signal.set(false);
                                            email_signal.set(String::default());
                                            password_signal.set(String::default());
                                            password2_signal.set(String::default());
                                            password_error_signal.set(String::default());
                                            email_error_signal.set(String::default());

                                        },
                                        MutationResult::Err(MutationError::UnableToRegisterUser) => {
                                            email_error_signal.set("Unable to register user".to_string());
                                            disabled_signal.set(false);
                                        },
                                        _ => {}
                                    }
                                } else {
                                    email_error_signal.set("Unable to register user".to_string());
                                    disabled_signal.set(false);
                                }
                            });
                        }
                    },
                    if !email_error_signal.read().is_empty() {
                        div {
                            class: r#"
                            text-sm
                            theme1:text-orange-300
                            theme2:text-teal-300
                            theme3:text-amber-400
                            "#,
                            "{email_error_signal.read()}"
                        }
                    }
                    Input {
                        r#type: "email",
                        name: "email",
                        placeholder: "Email",
                        oninput: move |e: Event<FormData>| {
                            email_signal.set(e.value().clone());
                        }
                    }
                    if !password_error_signal.read().is_empty() {
                        div {
                            class: r#"
                            text-sm
                            theme1:text-orange-300
                            theme2:text-teal-300
                            theme3:text-amber-400
                            "#,
                            "{password_error_signal.read()}"
                        }
                    }
                    Input {
                        r#type: "password",
                        name: "password",
                        placeholder: "Password",
                        oninput: move |e: Event<FormData>| {
                            password_signal.set(e.value().clone());
                        }
                    }
                    Input {
                        r#type: "password",
                        name: "password2",
                        placeholder: "Password again",
                        oninput: move |e: Event<FormData>| {
                            password2_signal.set(e.value().clone());
                        }
                    }
                    div {
                        ThemedButton {
                            disabled: Some(disabled_signal.read().clone()),
                            r#type: "submit",
                            "Register"
                        }
                    }
                }
            }
        }
    }
}

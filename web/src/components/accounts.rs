use std::ops::Deref;
use dioxus::prelude::*;
use dioxus_query::prelude::{use_mutation, use_query_client, MutationResult, MutationState};
use shared::{AuthenticatedUser, RegistrationUser};
use crate::env::APP_API_HOST;
use crate::cache::{MutationError, MutationValue, QueryError, QueryKey, QueryValue};
use crate::components::{Input, ThemedButton};
use crate::routes::Routes;
use crate::storage::{use_persistent, AppState};

async fn register_user(user: RegistrationUser) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = serde_json::json!({
        "username": user.username,
        "password": user.password,
    });

    let response = client.post(format!("{}/api/users", APP_API_HOST))
        .json(&json_body)
        .send().await;

    match response {
        Ok(response) => {
            match response.json::<AuthenticatedUser>().await {
                Ok(user) => {
                    MutationResult::Ok(MutationValue::User(user))
                }
                Err(_) => {
                    MutationResult::Err(MutationError::UnableToRegisterUser)
                }
            }
        }
        Err(e) => {
            dioxus_logger::tracing::error!("error creating user: {:?}", e);
            MutationResult::Err(MutationError::UnableToRegisterUser)
        }
    }
}

async fn authenticate_user(user: RegistrationUser) -> MutationResult<MutationValue, MutationError> {
    let client = reqwest::Client::new();
    let json_body = serde_json::json!({
        "username": user.username,
        "password": user.password,
    });

    let response = client.post(format!("{}/api/users/authenticate", APP_API_HOST))
        .json(&json_body)
        .send().await;

    match response {
        Ok(response) => {
            match response.json::<AuthenticatedUser>().await {
                Ok(user) => {
                    MutationResult::Ok(MutationValue::User(user))
                }
                Err(_) => {
                    MutationResult::Err(MutationError::UnableToAuthenticateUser)
                }
            }
        }
        Err(e) => {
            dioxus_logger::tracing::error!("error authenticating user: {:?}", e);
            MutationResult::Err(MutationError::UnableToAuthenticateUser)
        }
    }
}

#[component]
pub fn Accounts() -> Element {
    rsx! {
        div {
            class: "min-h-full grid grid-col gap-8",
            Outlet::<Routes> {}
        }
    }
}

#[component]
pub fn AccountsPage() -> Element {
    let storage = use_persistent("hangry-games", AppState::default);

    rsx! {
        if storage.get().jwt.is_some() {
            LogoutButton {}
        } else {
            div {
                class: r#"
                grid
                grid-cols-1
                sm:grid-cols-2
                gap-6
                "#,
                LoginForm {}
                RegisterForm {}
            }
            p {
                class: r#"
                text-sm
                text-center
                theme1:text-amber-200
                theme2:text-green-200
                theme3:text-stone-200
                "#,

                "Your username and password are stored in a secure database. We do not share your data with third parties."
            }
        }
    }
}

#[component]
fn LoginForm() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let mut username_signal = use_signal(String::default);
    let mut password_signal = use_signal(String::default);
    let mut disabled_signal = use_signal(|| false);
    let mut username_error_signal = use_signal(String::default);
    let mut password_error_signal = use_signal(String::default);

    let mutate = use_mutation(authenticate_user);

    let mut storage = use_persistent("hangry-games", AppState::default);

    rsx! {
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
                id: "register-form",
                method: "POST",
                onsubmit: move |_| {
                    username_error_signal.set(String::default());
                    password_error_signal.set(String::default());

                    let username = username_signal.read().clone();
                    if username.is_empty() {
                        username_error_signal.set("Username is required".to_string());
                    }

                    let password = password_signal.read().clone();
                    if password.is_empty() {
                        password_error_signal.set("Password is required".to_string());
                    }

                    if username_error_signal.peek().is_empty() && password_error_signal.peek().is_empty() {
                        disabled_signal.set(true);

                        spawn(async move {
                            let user = RegistrationUser {
                                username: username.clone(),
                                password: password.clone(),
                            };
                            mutate.mutate_async(user).await;
                            if let MutationState::Settled(Ok(result)) = mutate.result().deref() {
                                if let MutationValue::User(user) = result {
                                        client.invalidate_queries(&[QueryKey::User]);
                                        disabled_signal.set(false);
                                        username_signal.set(String::default());
                                        password_signal.set(String::default());
                                        password_error_signal.set(String::default());
                                        username_error_signal.set(String::default());

                                        let mut state = storage.get();
                                        state.jwt = Some(user.jwt.clone());
                                        state.username = Some(username.clone());
                                        storage.set(state);

                                        let navigator = use_navigator();
                                        navigator.replace(Routes::GamesList {});
                                }
                            } else if let MutationState::Settled(Err(err)) = mutate.result().deref() {
                                if let MutationError::UnableToAuthenticateUser = err {
                                        username_error_signal.set("Unable to authenticate user".to_string());
                                        disabled_signal.set(false);

                                }
                            }
                        });
                    }
                },
                if !username_error_signal.read().is_empty() {
                    div {
                        class: r#"
                        text-sm
                        theme1:text-orange-300
                        theme2:text-teal-300
                        theme3:text-amber-400
                        "#,
                        "{username_error_signal.read()}"
                    }
                }
                Input {
                    r#type: "string",
                    name: "username",
                    placeholder: "Username",
                    oninput: move |e: Event<FormData>| {
                        username_signal.set(e.value().clone());
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

#[component]
fn RegisterForm() -> Element {
    let client = use_query_client::<QueryValue, QueryError, QueryKey>();

    let mut username_signal = use_signal(String::default);
    let mut password_signal = use_signal(String::default);
    let mut password2_signal = use_signal(String::default);
    let mut disabled_signal = use_signal(|| false);
    let mut username_error_signal = use_signal(String::default);
    let mut password_error_signal = use_signal(String::default);

    let mutate = use_mutation(register_user);

    let mut storage = use_persistent("hangry-games", AppState::default);

    rsx! {
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
                    username_error_signal.set(String::default());
                    password_error_signal.set(String::default());

                    let username = username_signal.read().clone();
                    if username.is_empty() {
                        username_error_signal.set("Username is required".to_string());
                    }

                    let password = password_signal.read().clone();
                    let password2 = password2_signal.read().clone();

                    if password != password2 {
                        password_error_signal.set("Passwords do not match".to_string());
                    }

                    if username_error_signal.peek().is_empty() && password_error_signal.peek().is_empty() {
                        disabled_signal.set(true);

                        spawn(async move {
                            let user = RegistrationUser {
                                username: username.clone(),
                                password: password.clone(),
                            };
                            mutate.mutate_async(user).await;
                            match mutate.result().deref() {
                                MutationState::Settled(Ok(result)) => {
                                    if let MutationValue::User(user) = result {
                                        client.invalidate_queries(&[QueryKey::User]);
                                        disabled_signal.set(false);
                                        username_signal.set(String::default());
                                        password_signal.set(String::default());
                                        password2_signal.set(String::default());
                                        password_error_signal.set(String::default());
                                        username_error_signal.set(String::default());

                                        let mut state = storage.get();
                                        state.jwt = Some(user.jwt.clone());
                                        state.username = Some(username.clone());
                                        storage.set(state);

                                        let navigator = use_navigator();
                                        navigator.replace(Routes::GamesList {});
                                    }
                                },
                                MutationState::Settled(Err(err)) => {
                                    if let MutationError::UnableToRegisterUser = err {
                                        username_error_signal.set("Unable to register user".to_string());
                                    }
                                    disabled_signal.set(false);
                                },
                                _ => {}
                            }
                        });
                    }
                },
                if !username_error_signal.read().is_empty() {
                    div {
                        class: r#"
                        text-sm
                        theme1:text-orange-300
                        theme2:text-teal-300
                        theme3:text-amber-400
                        "#,
                        "{username_error_signal.read()}"
                    }
                }
                Input {
                    r#type: "text",
                    name: "username",
                    placeholder: "Username",
                    oninput: move |e: Event<FormData>| {
                        username_signal.set(e.value().clone());
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

#[component]
fn LogoutButton() -> Element {
    let mut storage = use_persistent("hangry-games", AppState::default);
    let state = storage.get();
    let username = state.username.clone().unwrap_or("whoever you are".to_string());

    rsx! {
        form {
            class: "flex flex-col gap-4 mt-4",
            onsubmit: move |_| {
                let mut state = storage.get();
                state.username = None;
                state.jwt = None;
                storage.set(state);
                let navigator = use_navigator();
                navigator.replace(Routes::Home {});
            },
            p {
                class: r#"
                text-xl
                text-center
                theme1:text-amber-500
                theme1:font-[Cinzel]
                theme2:text-green-300
                theme2:font-[Playfair_Display]
                theme3:text-stone-700
                theme3:font-[Orbitron]
                "#,
                "Thanks for playing, {username}!"
            }
            ThemedButton {
                class: "w-full",
                r#type: "submit",
                "Logout"
            }
        }
    }
}

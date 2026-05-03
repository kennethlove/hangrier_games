use crate::cache::MutationError;
use crate::components::{Input, ThemedButton};
use crate::env::APP_API_HOST;
use crate::http::WithCredentials;
use crate::routes::Routes;
use crate::storage::{AppState, use_persistent};
use dioxus::prelude::*;
use dioxus_query::prelude::*;
use shared::{AuthenticatedUser, RegistrationUser};
use validator::Validate;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct RegisterUserM;

impl MutationCapability for RegisterUserM {
    type Ok = AuthenticatedUser;
    type Err = MutationError;
    type Keys = RegistrationUser;

    async fn run(&self, user: &RegistrationUser) -> Result<AuthenticatedUser, MutationError> {
        let client = reqwest::Client::new();
        let json_body = serde_json::json!({
            "username": user.username,
            "password": user.password,
        });

        match client
            .post(format!("{}/api/users", APP_API_HOST))
            .with_credentials()
            .json(&json_body)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<AuthenticatedUser>().await {
                        Ok(user) => Ok(user),
                        Err(_) => Err(MutationError::UnableToRegisterUser),
                    }
                } else {
                    let body: serde_json::Value =
                        response.json().await.unwrap_or(serde_json::Value::Null);
                    let message = body
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Err(MutationError::RegistrationFailed { message })
                }
            }
            Err(e) => {
                tracing::error!("error creating user: {:?}", e);
                Err(MutationError::UnableToRegisterUser)
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct LoginUserM;

impl MutationCapability for LoginUserM {
    type Ok = AuthenticatedUser;
    type Err = MutationError;
    type Keys = RegistrationUser;

    async fn run(&self, user: &RegistrationUser) -> Result<AuthenticatedUser, MutationError> {
        let client = reqwest::Client::new();
        let json_body = serde_json::json!({
            "username": user.username,
            "password": user.password,
        });

        match client
            .post(format!("{}/api/users/authenticate", APP_API_HOST))
            .with_credentials()
            .json(&json_body)
            .send()
            .await
        {
            Ok(response) => match response.json::<AuthenticatedUser>().await {
                Ok(user) => Ok(user),
                Err(_) => Err(MutationError::UnableToAuthenticateUser),
            },
            Err(e) => {
                tracing::error!("error authenticating user: {:?}", e);
                Err(MutationError::UnableToAuthenticateUser)
            }
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
        if storage.get().username.is_some() {
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
    let mut username_signal = use_signal(String::default);
    let mut password_signal = use_signal(String::default);
    let mut disabled_signal = use_signal(|| false);
    let mut username_error_signal = use_signal(String::default);
    let mut password_error_signal = use_signal(String::default);

    let mutate = use_mutation(Mutation::new(LoginUserM));

    let mut storage = use_persistent("hangry-games", AppState::default);
    let navigator = use_navigator();

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
                onsubmit: move |evt: FormEvent| {
                    evt.prevent_default();
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
                            let reader = mutate.mutate_async(user).await;
                            let state = reader.state();
                            match &*state {
                                MutationStateData::Settled { res: Ok(_user), .. } => {
                                    disabled_signal.set(false);
                                    username_signal.set(String::default());
                                    password_signal.set(String::default());
                                    password_error_signal.set(String::default());
                                    username_error_signal.set(String::default());

                                    let mut state = storage.get();
                                    state.username = Some(username.clone());
                                    storage.set(state);

                                    navigator.replace(Routes::GamesList {});
                                }
                                MutationStateData::Settled { res: Err(MutationError::UnableToAuthenticateUser), .. } => {
                                    username_error_signal.set("Unable to authenticate user".to_string());
                                    disabled_signal.set(false);
                                }
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
    let mut username_signal = use_signal(String::default);
    let mut password_signal = use_signal(String::default);
    let mut password2_signal = use_signal(String::default);
    let mut disabled_signal = use_signal(|| false);
    let mut username_error_signal = use_signal(String::default);
    let mut password_error_signal = use_signal(String::default);

    let mutate = use_mutation(Mutation::new(RegisterUserM));

    let mut storage = use_persistent("hangry-games", AppState::default);
    let navigator = use_navigator();

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
                onsubmit: move |evt: FormEvent| {
                    evt.prevent_default();
                    username_error_signal.set(String::default());
                    password_error_signal.set(String::default());

                    let username = username_signal.read().clone();
                    let password = password_signal.read().clone();
                    let password2 = password2_signal.read().clone();

                    let user = RegistrationUser {
                        username: username.clone(),
                        password: password.clone(),
                    };
                    if let Err(errs) = user.validate() {
                        for (field, ferrs) in errs.field_errors() {
                            let msg = ferrs
                                .iter()
                                .filter_map(|e| e.message.as_ref())
                                .next()
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| format!("{field} is invalid"));
                            match field.as_ref() {
                                "username" => username_error_signal.set(msg),
                                "password" => password_error_signal.set(msg),
                                _ => {}
                            }
                        }
                    }

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
                            let reader = mutate.mutate_async(user).await;
                            let state = reader.state();
                            match &*state {
                                MutationStateData::Settled { res: Ok(_user), .. } => {
                                    disabled_signal.set(false);
                                    username_signal.set(String::default());
                                    password_signal.set(String::default());
                                    password2_signal.set(String::default());
                                    password_error_signal.set(String::default());
                                    username_error_signal.set(String::default());

                                    let mut state = storage.get();
                                    state.username = Some(username.clone());
                                    storage.set(state);

                                    navigator.replace(Routes::GamesList {});
                                }
                                MutationStateData::Settled { res: Err(err), .. } => {
                                    match err {
                                        MutationError::RegistrationFailed { message } => {
                                            let lowered = message.to_lowercase();
                                            if lowered.contains("password") {
                                                password_error_signal.set(message.clone());
                                            } else if !message.is_empty() {
                                                username_error_signal.set(message.clone());
                                            } else {
                                                username_error_signal
                                                    .set("Unable to register user".to_string());
                                            }
                                        }
                                        _ => {
                                            username_error_signal
                                                .set("Unable to register user".to_string());
                                        }
                                    }
                                    disabled_signal.set(false);
                                }
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
                        disabled: Some(*disabled_signal.read()),
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
    let username = state
        .username
        .clone()
        .unwrap_or("whoever you are".to_string());
    let navigator = use_navigator();

    rsx! {
        form {
            class: "flex flex-col gap-4 mt-4",
            onsubmit: move |evt: FormEvent| {
                evt.prevent_default();
                let mut state = storage.get();
                state.username = None;
                storage.set(state);

                spawn(async move {
                    let _ = reqwest::Client::new()
                        .post(format!("{}/api/auth/logout", APP_API_HOST))
                        .with_credentials()
                        .send()
                        .await;
                });

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

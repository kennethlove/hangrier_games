use std::sync::LazyLock;
use tera::{Context, Tera};

use crate::templates::AuthState;
use crate::templates::filters;

pub static TERA: LazyLock<Tera> = LazyLock::new(|| {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let pattern = format!("{}/templates/**/*.html", manifest_dir);
    tracing::info!("Loading Tera templates from: {}", pattern);
    let mut tera = match Tera::new(&pattern) {
        Ok(t) => {
            let names: Vec<&str> = t.get_template_names().collect();
            tracing::info!("Loaded {} Tera templates: {:?}", names.len(), names);
            t
        }
        Err(e) => {
            tracing::error!("Tera template init failed: {e}");
            panic!("Tera template init failed: {e}");
        }
    };
    tera.register_filter("icon", filters::icon_filter);
    tera.register_filter("narrative_icon", filters::narrative_icon_filter);
    tera.register_filter("status_color", filters::status_color);
    tera.register_filter("hunger_label", filters::hunger_label);
    tera.register_filter("hunger_color", filters::hunger_color);
    tera.register_filter("thirst_label", filters::thirst_label);
    tera.register_filter("thirst_color", filters::thirst_color);
    tera.register_filter("stamina_label", filters::stamina_label);
    tera.register_filter("stamina_color", filters::stamina_color);
    tera.register_filter("message_kind", filters::message_kind);
    tera.register_filter("archetype_label", filters::archetype_label);
    tera.register_filter("kind_color", filters::kind_color);
    tera.register_filter("format_words", filters::format_words);
    tera.register_filter("upper", filters::upper);
    tera.register_filter("lower", filters::lower);
    tera.register_filter("phase_label", filters::phase_label);
    tera.register_filter("phase_class", filters::phase_class);
    tera.register_filter("json", filters::json);
    tera
});

pub fn render(template: &str, ctx: &Context) -> String {
    TERA.render(template, ctx)
        .unwrap_or_else(|e| format!("Template error: {e}"))
}

pub fn auth_context(auth: &AuthState) -> (bool, String, String) {
    match auth {
        AuthState::Authenticated {
            username,
            csrf_token,
            ..
        } => (true, username.clone(), csrf_token.clone()),
        AuthState::Guest { csrf_token } => (false, String::new(), csrf_token.clone()),
    }
}

pub fn base_context(title: &str, auth: &AuthState) -> Context {
    let (authenticated, username, csrf_token) = auth_context(auth);
    let mut ctx = Context::new();
    ctx.insert("title", title);
    ctx.insert("auth_authenticated", &authenticated);
    ctx.insert("auth_username", &username);
    ctx.insert("csrf_token", &csrf_token);
    ctx
}

use maud::{DOCTYPE, Markup, PreEscaped, html};

pub mod auth;
pub mod game_detail;
pub mod pages;
pub mod timeline;

/// Authentication state passed to templates for conditional rendering.
#[derive(Clone, Default)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub username: Option<String>,
}

impl AuthState {
    pub fn authenticated(username: impl Into<String>) -> Self {
        Self {
            is_authenticated: true,
            username: Some(username.into()),
        }
    }

    pub fn guest() -> Self {
        Self {
            is_authenticated: false,
            username: None,
        }
    }
}

pub fn auth_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Hangry Games" }
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link href="https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,16..72,200..800;1,16..72,200..800&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet";
                style {
                    (PreEscaped(r#"
:root {
  --bg: oklch(97.5% 0.012 75);
  --surface: oklch(99.5% 0.004 75);
  --fg: oklch(18% 0.015 70);
  --muted: oklch(52% 0.012 70);
  --border: oklch(88% 0.01 75);
  --accent: oklch(42% 0.18 285);
  --accent-soft: color-mix(in oklch, var(--accent) 10%, transparent);
  --fg-soft: color-mix(in oklch, var(--fg) 5%, transparent);
  --font-display: 'Newsreader', 'Iowan Old Style', Georgia, serif;
  --font-body: 'Inter', -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', ui-monospace, Menlo, monospace;
  --fs-h2: 28px;
  --fs-body: 15px;
  --fs-meta: 12px;
  --fs-xs: 11px;
  --gap-xs: 6px;
  --gap-sm: 12px;
  --gap-md: 20px;
  --gap-lg: 32px;
  --container: 440px;
  --radius-sm: 4px;
  --radius: 8px;
}
@media (prefers-color-scheme: dark) {
  :root {
    --bg: oklch(16% 0.02 280);
    --surface: oklch(20% 0.02 280);
    --fg: oklch(88% 0.01 75);
    --muted: oklch(58% 0.015 280);
    --border: oklch(28% 0.02 280);
    --accent: oklch(62% 0.22 285);
    --accent-soft: color-mix(in oklch, var(--accent) 15%, transparent);
    --fg-soft: color-mix(in oklch, var(--fg) 8%, transparent);
  }
}
[data-theme="dark"] {
  --bg: oklch(16% 0.02 280);
  --surface: oklch(20% 0.02 280);
  --fg: oklch(88% 0.01 75);
  --muted: oklch(58% 0.015 280);
  --border: oklch(28% 0.02 280);
  --accent: oklch(62% 0.22 285);
  --accent-soft: color-mix(in oklch, var(--accent) 15%, transparent);
  --fg-soft: color-mix(in oklch, var(--fg) 8%, transparent);
}
*, *::before, *::after { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body {
  margin: 0; background: var(--bg); color: var(--fg);
  font-family: var(--font-body); font-size: var(--fs-body);
  line-height: 1.6; min-height: 100vh;
  display: flex; align-items: center; justify-content: center;
  padding: var(--gap-lg);
}
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
button { font: inherit; cursor: pointer; }
.auth-card {
  width: 100%; max-width: var(--container);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  padding: var(--gap-lg);
}
.auth-logo {
  font-family: var(--font-display);
  font-size: 22px; font-weight: 700;
  letter-spacing: 0.06em; text-transform: uppercase;
  text-align: center; margin-bottom: var(--gap-lg);
}
.auth-logo span { color: var(--accent); }
.auth-title {
  font-family: var(--font-display);
  font-size: var(--fs-h2); font-weight: 700;
  margin: 0 0 4px; letter-spacing: -0.01em;
}
.auth-subtitle {
  font-size: var(--fs-meta); color: var(--muted);
  margin: 0 0 var(--gap-md);
}
.form-group { margin-bottom: var(--gap-sm); }
.form-group label {
  display: block; font-size: var(--fs-meta); font-weight: 500;
  color: var(--fg); margin-bottom: 4px;
}
.form-group input {
  width: 100%; padding: 10px 12px;
  border: 1px solid var(--border); border-radius: var(--radius-sm);
  font-family: var(--font-body); font-size: var(--fs-body);
  background: var(--bg); color: var(--fg); outline: none;
  transition: border-color 0.12s, box-shadow 0.12s;
}
.form-group input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px var(--accent-soft);
}
.form-row {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: var(--gap-md); font-size: var(--fs-meta);
}
.form-row label {
  display: flex; align-items: center; gap: 6px;
  color: var(--muted); cursor: pointer;
}
.btn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 100%; padding: 12px 24px;
  border-radius: var(--radius-sm); border: none;
  font-size: 15px; font-weight: 600;
  cursor: pointer; transition: opacity 0.15s;
}
.btn:hover { opacity: 0.85; }
.btn-primary { background: var(--accent); color: #fff; }
.btn-ghost { background: transparent; color: var(--fg); border: 1px solid var(--border); }
.auth-footer {
  text-align: center; margin-top: var(--gap-md);
  font-size: var(--fs-meta); color: var(--muted);
}
.divider {
  display: flex; align-items: center; gap: var(--gap-sm);
  margin: var(--gap-md) 0;
  font-size: var(--fs-xs); color: var(--muted);
  text-transform: uppercase; letter-spacing: 0.04em;
}
.divider::before, .divider::after {
  content: ''; flex: 1; height: 1px; background: var(--border);
}
.tab-bar {
  display: flex; gap: 2px; margin-bottom: var(--gap-lg);
  background: var(--bg); border-radius: var(--radius-sm); padding: 2px;
}
.tab-btn {
  flex: 1; padding: 8px; border: none; background: transparent;
  font-size: var(--fs-meta); font-weight: 500; color: var(--muted);
  border-radius: var(--radius-sm); cursor: pointer;
  transition: background 0.12s, color 0.12s;
}
.tab-btn.active {
  background: var(--surface); color: var(--fg);
  box-shadow: 0 1px 3px color-mix(in oklch, var(--fg) 8%, transparent);
}
.tab-panel { display: none; }
.tab-panel.active { display: block; }
.error-banner {
  margin-bottom: var(--gap-md); padding: 10px 12px;
  border: 1px solid oklch(60% 0.22 25); border-radius: var(--radius-sm);
  background: oklch(60% 0.22 25 / 0.1); color: oklch(60% 0.22 25);
  font-size: var(--fs-meta);
}
.theme-toggle-btn {
  position: fixed; top: 12px; right: 12px;
  background: transparent; border: 1px solid var(--border); border-radius: var(--radius-sm);
  padding: 4px 8px; font-size: 14px; cursor: pointer; color: var(--muted);
  transition: background 0.12s, color 0.12s;
}
.theme-toggle-btn:hover { background: var(--fg-soft); color: var(--fg); }
                    "#))
                }
            }
            body {
                button class="theme-toggle-btn" aria-label="Toggle theme" { "🌙" }
                div class="auth-card" {
                    div class="auth-logo" { "Hangry " span { "Games" } }
                    div class="tab-bar" {
                        button class="tab-btn active" data-tab="login" { "Sign In" }
                        button class="tab-btn" data-tab="register" { "Register" }
                        button class="tab-btn" data-tab="reset" { "Reset Password" }
                    }
                    (content)
                }
                script {
                    (PreEscaped(r#"
function switchTab(id) {
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
  document.querySelector('[data-tab="'+id+'"]').classList.add('active');
  document.getElementById(id).classList.add('active');
}
document.querySelectorAll('.tab-btn').forEach(btn => {
  btn.addEventListener('click', () => switchTab(btn.dataset.tab));
});
const themeBtn = document.querySelector('.theme-toggle-btn');
if (themeBtn) {
  themeBtn.addEventListener('click', () => {
    const root = document.documentElement;
    const isDark = root.getAttribute('data-theme') === 'dark';
    root.setAttribute('data-theme', isDark ? 'light' : 'dark');
    themeBtn.textContent = isDark ? '🌙' : '☀️';
  });
}
                    "#))
                }
            }
        }
    }
}

pub fn base_layout(title: &str, auth: AuthState, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " — Hangry Games" }
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link href="https://fonts.googleapis.com/css2?family=Newsreader:ital,opsz,wght@0,16..72,200..800;1,16..72,200..800&family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet";
                link rel="stylesheet" href="/assets/main.css";
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                script src="https://unpkg.com/htmx-ext-sse@2.2.3" {}
            }
            body {
                // SVG sprites served as static files
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-ui.svg"/></svg>"#))
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" style="display:none"><use href="/icons/sprite-narrative.svg"/></svg>"#))
                header class="topnav" {
                    div class="container topnav-inner" {
                        div class="logo" {
                            "Hangry "
                            span { "Games" }
                        }
                        nav {
                            a href="/games" class="active" { "Broadcast" }
                            a href="#" { "Tributes" }
                            a href="#" { "Arena" }
                            a href="#" { "Odds" }
                        }
                        (auth_links(&auth))
                    }
                }
                main {
                    (content)
                }
                footer class="pagefoot" {
                    div class="container row-between" {
                        span { "© Hangry Games" }
                        span class="num" { "Server v0.1.15" }
                    }
                }
            }
        }
    }
}

/// Render auth links based on authentication state.
fn auth_links(auth: &AuthState) -> Markup {
    html! {
        div class="auth-links" {
            @if auth.is_authenticated {
                a href="/account" { (auth.username.as_deref().unwrap_or("Account")) }
            } @else {
                a href="/auth" { "Login" }
            }
        }
    }
}

pub fn icon(name: &str) -> Markup {
    html! {
        svg class="icon" {
            use href=(format!("#icon_ui_{}", name)) {}
        }
    }
}

pub fn narrative_icon(name: &str) -> Markup {
    html! {
        svg class="icon" {
            use href=(format!("#icon_narrative_{}", name)) {}
        }
    }
}

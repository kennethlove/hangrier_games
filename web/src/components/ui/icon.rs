use dioxus::prelude::*;

/// Tier of an icon. Determines default size and signals visual register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconTier {
    /// Geometric, Heroicons-style. 24x24 source viewBox.
    Ui,
    /// Illustrative, game-icons.net-style. 512x512 source viewBox.
    Narrative,
}

/// Standard icon size token. Maps to a Tailwind `size-*` utility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

impl IconSize {
    /// Tailwind utility class to apply to the rendered `<svg>`.
    pub fn class(self) -> &'static str {
        match self {
            IconSize::Xs => "size-3",
            IconSize::Sm => "size-4",
            IconSize::Md => "size-5",
            IconSize::Lg => "size-8",
            IconSize::Xl => "size-12",
            IconSize::Xxl => "size-20",
        }
    }
}

impl IconTier {
    /// Default size token for this tier.
    pub fn default_size(self) -> IconSize {
        match self {
            IconTier::Ui => IconSize::Sm,
            IconTier::Narrative => IconSize::Lg,
        }
    }
}

/// Render an icon by referencing the inlined sprite via `<use>`.
///
/// Color comes from `currentColor` (i.e. parent `text-*` utility classes).
/// Pass an `aria_label` to make the icon meaningful; otherwise it is decorative
/// (`aria-hidden="true"`).
#[component]
pub fn Icon(
    /// Sprite ID to reference (e.g. `"ui-edit"` or `"narrative-fist"`).
    /// Use the codegen wrappers (`EditIcon`, `FistIcon`, ...) instead of
    /// constructing this primitive directly when the icon name is static.
    sprite_id: String,
    /// Source viewBox of the symbol. UI icons are `"0 0 24 24"`; narrative
    /// icons are `"0 0 512 512"`.
    view_box: String,
    /// Size token; pass `None` to use the wrapper's default.
    #[props(default)]
    size: Option<IconSize>,
    tier: IconTier,
    #[props(default)] class: String,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let size_class = size.unwrap_or_else(|| tier.default_size()).class();
    let combined_class = if class.is_empty() {
        size_class.to_string()
    } else {
        format!("{size_class} {class}")
    };
    let href = format!("#{sprite_id}");

    rsx! {
        if let Some(label) = aria_label {
            svg {
                class: "{combined_class}",
                view_box: "{view_box}",
                role: "img",
                "aria-label": "{label}",
                "focusable": "false",
                r#use { href: "{href}" }
            }
        } else {
            svg {
                class: "{combined_class}",
                view_box: "{view_box}",
                "aria-hidden": "true",
                "focusable": "false",
                r#use { href: "{href}" }
            }
        }
    }
}

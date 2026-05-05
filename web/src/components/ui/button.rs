use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Ghost,
    Danger,
    Chrome,
}

impl ButtonVariant {
    pub fn classes(self) -> &'static str {
        match self {
            ButtonVariant::Primary =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-primary text-bg",
            ButtonVariant::Ghost =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-transparent text-primary ring-1 ring-inset ring-primary",
            ButtonVariant::Danger =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-inner font-text font-bold text-xs uppercase tracking-[0.12em] cursor-pointer bg-danger text-white",
            ButtonVariant::Chrome =>
                "inline-flex items-center gap-1.5 px-4 py-2 rounded-none font-display text-sm tracking-wider cursor-pointer bg-surface-2 text-text",
        }
    }
}

#[component]
pub fn Button(
    #[props(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[props(default = false)] disabled: bool,
    #[props(default)] onclick: EventHandler<MouseEvent>,
    children: Element,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "{variant.classes()}",
            disabled,
            onclick: move |evt| onclick.call(evt),
            {children}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_classes_are_distinct() {
        assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Ghost.classes());
        assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Danger.classes());
        assert_ne!(ButtonVariant::Primary.classes(), ButtonVariant::Chrome.classes());
    }

    #[test]
    fn chrome_variant_has_no_radius() {
        assert!(ButtonVariant::Chrome.classes().contains("rounded-none"));
    }

    #[test]
    fn primary_variant_uses_primary_color() {
        assert!(ButtonVariant::Primary.classes().contains("bg-primary"));
    }
}

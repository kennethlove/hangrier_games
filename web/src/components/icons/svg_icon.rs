use dioxus::prelude::*;

/// Lazy-loaded SVG icon component
///
/// Uses SVG <use> elements to reference icons from a sprite sheet.
/// The sprite sheet should be loaded via <link> or script in index.html
/// The sprite sheet at /assets/icons.svg contains all icons as symbols.
#[component]
pub fn SvgIcon(name: String, class: String) -> Element {
    let icon_id = format!("icon-{}", name);

    rsx! {
        svg {
            class,
            view_box: "0 0 512 512",
            xmlns: "http://www.w3.org/2000/svg",
            use {
                "xlink:href": format!("#{}", icon_id),
            }
        }
    }
}

/// Component that loads the SVG sprite sheet via JavaScript
///
/// This injects the sprite sheet into the DOM so icons can be referenced.
/// It loads asynchronously to avoid blocking the initial render.
#[component]
pub fn SpriteSheetLoader() -> Element {
    // Generate JavaScript to fetch and inject the sprite sheet
    let js_code = r#"
        (function() {
            var container = document.getElementById('svg-sprite-container');
            if (container) return;
            
            container = document.createElement('div');
            container.id = 'svg-sprite-container';
            container.style.display = 'none';
            document.body.appendChild(container);
            
            fetch('/assets/icons.svg')
                .then(function(response) { return response.text(); })
                .then(function(svg) {
                    container.innerHTML = svg;
                })
                .catch(function(e) {
                    console.error('Failed to load sprite sheet:', e);
                });
        })();
    "#;

    rsx! {
        script { dangerous_inner_html: js_code }
    }
}

/// Get the icon name for an item based on its type and attribute
pub fn icon_name_for_item(item: &game::items::Item) -> String {
    use game::items::{Attribute, ItemType};

    match item.item_type {
        ItemType::Consumable => match item.attribute {
            Attribute::Health => "health_potion",
            Attribute::Sanity => "spinning_top",
            Attribute::Movement => "trail_mix",
            Attribute::Bravery => "powder",
            Attribute::Speed => "fizzing_flask",
            Attribute::Strength => "hypodermic_test",
            Attribute::Defense => "spray",
        },
        ItemType::Food(_) => "trail_mix",
        ItemType::Water(_) => "fizzing_flask",
        ItemType::Weapon => match item.attribute {
            Attribute::Strength => {
                let name = item.to_string().to_lowercase();
                let weapon_name = name.rsplit_once(' ').map(|(_, w)| w).unwrap_or(&name);

                match weapon_name {
                    "sword" => "pointy_sword",
                    "spear" => "spear_hook",
                    "dagger" => "plain_dagger",
                    "knife" => "switchblade",
                    "net" => "fishing_net",
                    "trident" => "harpoon_trident",
                    "bow" => "high_shot",
                    "mace" => "spiked_mace",
                    "axe" => "wood_axe",
                    _ => "fist",
                }
            }
            Attribute::Defense => "shield",
            _ => "fist",
        },
    }
    .to_string()
}

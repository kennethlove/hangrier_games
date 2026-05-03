use dioxus::prelude::*;
use game::areas::AreaDetails;
use game::areas::forage::forage_richness;
use game::areas::shelter::shelter_quality;
use game::areas::water::water_source;
use game::areas::weather::current_weather;

#[component]
pub fn MapAffordanceOverlay(cx: f64, cy: f64, size: f64, area: AreaDetails) -> Element {
    let weather = current_weather();
    let terrain = area.terrain.base;
    let water = water_source(terrain, &weather);
    let forage = forage_richness(terrain);
    let shelter = shelter_quality(terrain, &weather);

    let glyph_size = size * 0.20;
    let row_y = cy + size * 0.55;
    let mut glyphs: Vec<&'static str> = Vec::new();
    if water > 0 {
        glyphs.push("💧");
    }
    if forage > 0 {
        glyphs.push("🌿");
    }
    if shelter >= 2 {
        glyphs.push("🏠");
    }
    if glyphs.is_empty() {
        return rsx! {};
    }
    let total = glyphs.len() as f64;
    let spacing = glyph_size * 1.2;
    let start_x = cx - ((total - 1.0) * spacing) / 2.0;

    rsx! {
        g {
            class: "pointer-events-none",
            for (i, g) in glyphs.into_iter().enumerate() {
                {
                    let x = start_x + (i as f64) * spacing;
                    rsx! {
                        text {
                            key: "{i}",
                            x: "{x}",
                            y: "{row_y}",
                            text_anchor: "middle",
                            font_size: "{glyph_size}",
                            "{g}"
                        }
                    }
                }
            }
        }
    }
}

use dioxus::prelude::*;
use game::areas::AreaDetails;
use game::areas::hex::{SUB_SIZE_RATIO, SUB_SLOTS, default_layout};

const HEX_SIZE: f64 = 90.0;
const PADDING: f64 = 16.0;

fn hex_corners(cx: f64, cy: f64, size: f64) -> String {
    // Pointy-top: corners at 30, 90, 150, 210, 270, 330 degrees.
    let mut pts = String::new();
    for i in 0..6 {
        let angle_deg = 60.0 * (i as f64) + 30.0;
        let a = angle_deg.to_radians();
        let x = cx + size * a.cos();
        let y = cy + size * a.sin();
        if i > 0 {
            pts.push(' ');
        }
        pts.push_str(&format!("{x:.2},{y:.2}"));
    }
    pts
}

#[component]
pub fn Map(areas: Vec<AreaDetails>) -> Element {
    let layout = default_layout();

    // Compute viewBox bounds.
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    for (_a, ax) in layout.iter() {
        let (x, y) = ax.to_pixel(HEX_SIZE);
        min_x = min_x.min(x - HEX_SIZE);
        max_x = max_x.max(x + HEX_SIZE);
        min_y = min_y.min(y - HEX_SIZE);
        max_y = max_y.max(y + HEX_SIZE);
    }
    let vb_x = min_x - PADDING;
    let vb_y = min_y - PADDING;
    let vb_w = (max_x - min_x) + 2.0 * PADDING;
    let vb_h = (max_y - min_y) + 2.0 * PADDING;
    let view_box = format!("{vb_x:.2} {vb_y:.2} {vb_w:.2} {vb_h:.2}");

    rsx! {
        svg {
            view_box: "{view_box}",
            for (i, (area, ax)) in layout.iter().enumerate() {
                {
                    let (cx, cy) = ax.to_pixel(HEX_SIZE);
                    let area_name = area.to_string();
                    let is_open = areas
                        .iter()
                        .find(|ad| ad.area == Some(*area))
                        .map(|ad| ad.is_open())
                        .unwrap_or(true);
                    let points = hex_corners(cx, cy, HEX_SIZE);
                    let area_id = area_name.to_lowercase().replace(' ', "-");
                    let label = format!("{}", i);
                    let on_click = move |_| {
                        tracing::info!("hex tile clicked: {}", area_name);
                    };
                    rsx! {
                        g {
                            key: "{area_id}",
                            onclick: on_click,
                            polygon {
                                id: "{area_id}",
                                "data-open": "{is_open}",
                                class: "fill-stone-200 data-[open=false]:fill-red-500 theme3:fill-stone-400 stroke-stone-700",
                                points: "{points}",
                                stroke_width: "2",
                            }
                            // Sub-tile grid: 7 smaller hexes per area for
                            // tribute positioning (presentation-only).
                            for (sub_i, slot) in SUB_SLOTS.iter().enumerate() {
                                {
                                    let sub_size = HEX_SIZE * SUB_SIZE_RATIO;
                                    let (sx_off, sy_off) = slot.to_pixel(sub_size);
                                    let sx = cx + sx_off;
                                    let sy = cy + sy_off;
                                    let sub_points = hex_corners(sx, sy, sub_size);
                                    let sub_id = format!("{area_id}-sub-{sub_i}");
                                    rsx! {
                                        polygon {
                                            key: "{sub_id}",
                                            id: "{sub_id}",
                                            class: "fill-transparent stroke-stone-500/40 pointer-events-none",
                                            points: "{sub_points}",
                                            stroke_width: "1",
                                            stroke_dasharray: "2 2",
                                        }
                                    }
                                }
                            }
                            text {
                                x: "{cx}",
                                y: "{cy}",
                                text_anchor: "middle",
                                dominant_baseline: "central",
                                class: "fill-stone-900 select-none pointer-events-none",
                                font_size: "32",
                                font_weight: "bold",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

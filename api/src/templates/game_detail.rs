use shared::messages::Phase;
use shared::{DisplayGame, GameStatus};

/// Determine the phase label and class from game state and messages.
pub fn current_broadcast_phase(
    game: &DisplayGame,
    messages: &[shared::messages::GameMessage],
) -> (&'static str, &'static str) {
    match game.status {
        GameStatus::Finished => ("finished", "FINISHED"),
        GameStatus::NotStarted => ("day", "STAGING"),
        GameStatus::InProgress => {
            if let Some(last) = messages.last() {
                match last.phase {
                    Phase::Dawn => ("dawn", "DAWN"),
                    Phase::Day => ("day", "DAY"),
                    Phase::Dusk => ("dusk", "DUSK"),
                    Phase::Night => ("night", "NIGHT"),
                }
            } else {
                ("day", "DAY")
            }
        }
    }
}

fn message_archetype(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "death",
        MessageKind::Combat | MessageKind::CombatSwing => "action",
        MessageKind::Alliance => "commentary",
        MessageKind::Movement => "commentary",
        MessageKind::Item | MessageKind::SponsorGift => "event",
        MessageKind::State => "commentary",
        MessageKind::Trauma => "commentary",
        MessageKind::Affliction => "commentary",
        MessageKind::Phobia => "commentary",
        MessageKind::Fixation => "commentary",
        MessageKind::Trapped => "event",
        MessageKind::Sleep => "commentary",
    }
}

fn archetype_label(archetype: &str) -> &'static str {
    match archetype {
        "action" => "ACTION",
        "death" => "ELIMINATED",
        "event" => "ARENA EVENT",
        "commentary" => "ANALYSIS",
        _ => "EVENT",
    }
}

fn message_kind_label(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "Death",
        MessageKind::Combat => "Combat",
        MessageKind::CombatSwing => "Combat",
        MessageKind::Alliance => "Alliance",
        MessageKind::Movement => "Movement",
        MessageKind::Item => "Item",
        MessageKind::SponsorGift => "Sponsor",
        MessageKind::State => "State",
        MessageKind::Trauma => "Trauma",
        MessageKind::Affliction => "Health",
        MessageKind::Phobia => "Fear",
        MessageKind::Fixation => "Fixation",
        MessageKind::Trapped => "Trapped",
        MessageKind::Sleep => "Sleep",
    }
}

fn kind_color(payload: &shared::messages::MessagePayload) -> &'static str {
    use shared::messages::MessageKind;
    match payload.kind() {
        MessageKind::Death => "var(--danger)",
        MessageKind::Combat | MessageKind::CombatSwing => "var(--waiting)",
        MessageKind::Alliance => "var(--info)",
        MessageKind::Movement => "var(--accent)",
        MessageKind::Item | MessageKind::SponsorGift => "var(--gold)",
        MessageKind::State => "var(--muted)",
        MessageKind::Trauma => "var(--purple)",
        MessageKind::Affliction => "var(--warning)",
        MessageKind::Phobia => "var(--purple)",
        MessageKind::Fixation => "var(--purple)",
        MessageKind::Trapped => "var(--warning)",
        MessageKind::Sleep => "var(--info)",
    }
}

fn hunger_label(hunger: u8) -> &'static str {
    match hunger {
        0 => "Sated",
        1 => "Hungry",
        _ => "Starving",
    }
}

fn hunger_color(hunger: u8) -> &'static str {
    match hunger {
        0 => "var(--running)",
        1 => "var(--waiting)",
        _ => "var(--danger)",
    }
}

fn thirst_label(thirst: u8) -> &'static str {
    match thirst {
        0 => "Hydrated",
        1 => "Thirsty",
        _ => "Dehydrated",
    }
}

fn thirst_color(thirst: u8) -> &'static str {
    match thirst {
        0 => "var(--running)",
        1 => "var(--waiting)",
        _ => "var(--danger)",
    }
}

fn stamina_label(stamina: u32, max_stamina: u32) -> &'static str {
    if max_stamina == 0 {
        return "Unknown";
    }
    let ratio = stamina as f64 / max_stamina as f64;
    if ratio > 0.66 {
        "Fresh"
    } else if ratio > 0.33 {
        "Winded"
    } else {
        "Exhausted"
    }
}

fn stamina_color(stamina: u32, max_stamina: u32) -> &'static str {
    if max_stamina == 0 {
        return "band-none";
    }
    let ratio = stamina as f64 / max_stamina as f64;
    if ratio > 0.66 {
        "band-good"
    } else if ratio > 0.33 {
        "band-warn"
    } else {
        "band-danger"
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn render_event_card(msg: &shared::messages::GameMessage) -> String {
    let archetype = message_archetype(&msg.payload);
    let badge = archetype_label(archetype);
    let kind = message_kind_label(&msg.payload);
    let color = kind_color(&msg.payload);
    let ts = format!(
        "D{} {} T{}",
        msg.game_day,
        format!("{:?}", msg.phase).to_uppercase(),
        msg.tick
    );

    format!(
        r#"<div class="event-card {archetype}" data-archetype="{archetype}" style="border-left-color:{color};">
          <div class="card-head">
            <span class="card-badge" style="background:{color};">{badge}</span>
            <span class="card-timestamp">{ts}</span>
          </div>
          <div class="card-body">
            <span style="font-weight:600;">{kind}</span>
            — {content}
          </div>
        </div>"#,
        content = html_escape(&msg.content)
    )
}

pub fn render_commentary_card(seg: &announcers::CommentarySegment) -> String {
    let speaker = seg
        .lines
        .first()
        .map(|l| l.speaker.as_str())
        .unwrap_or("COMMENTATOR");
    let text: String = seg
        .lines
        .iter()
        .map(|l| format!("<div>{}</div>", html_escape(&l.text)))
        .collect();

    format!(
        r#"<div class="event-card commentary" data-archetype="commentary">
          <div class="card-head">
            <span class="card-badge">ANALYSIS</span>
            <span class="card-timestamp">Day {} {}</span>
          </div>
          <div class="card-body">
            <div class="comment-text">{text}</div>
            <div class="comment-speaker">
              <div class="speaker-avatar">C</div>
              <div class="speaker-info">
                <span class="speaker-name">{}</span>
                <span class="speaker-role">ANALYSIS</span>
              </div>
            </div>
          </div>
        </div>"#,
        seg.day,
        seg.phase,
        html_escape(speaker)
    )
}

pub fn render_tribute_row(tribute: &game::tributes::Tribute) -> String {
    let is_alive = tribute.is_alive();
    let health = tribute.attributes.health;
    let health_class = if health > 60 {
        "high"
    } else if health > 20 {
        "mid"
    } else if health > 0 {
        "low"
    } else {
        "empty"
    };
    let initial = tribute.name.chars().next().unwrap_or('?');
    let avatar_color = if is_alive {
        "var(--broad-accent)"
    } else {
        "var(--broad-danger)"
    };
    let status_class = if is_alive { "alive" } else { "dead" };
    let status_text = if is_alive { "ALIVE" } else { "DEAD" };
    let dead_class = if !is_alive { " dead" } else { "" };

    format!(
        r#"<div class="roster-row{dead_class}">
          <div class="roster-avatar" style="border-color:{avatar_color};color:{avatar_color}">{initial}</div>
          <div class="roster-info">
            <span class="roster-name">{name}</span>
            <span class="roster-district">D{district}</span>
          </div>
          <div class="roster-health">
            <div class="roster-health-bar">
              <div class="roster-health-fill {health_class}" style="width:{health}%;"></div>
            </div>
          </div>
          <span class="roster-status {status_class}">{status_text}</span>
        </div>"#,
        name = html_escape(&tribute.name),
        district = tribute.district,
    )
}

pub fn render_tribute_card(tribute: &game::tributes::Tribute) -> String {
    let is_alive = tribute.is_alive();
    let health = tribute.attributes.health;
    let _health_class = if health > 60 {
        "high"
    } else if health > 20 {
        "mid"
    } else if health > 0 {
        "low"
    } else {
        "empty"
    };
    let status_class = if is_alive { "alive" } else { "dead" };
    let status_text = if is_alive { "ALIVE" } else { "DEAD" };
    let hunger = hunger_label(tribute.hunger);
    let hunger_c = hunger_color(tribute.hunger);
    let thirst = thirst_label(tribute.thirst);
    let thirst_c = thirst_color(tribute.thirst);
    let stamina = stamina_label(tribute.stamina, tribute.max_stamina);
    let stamina_c = stamina_color(tribute.stamina, tribute.max_stamina);

    format!(
        r#"<div class="tribute-card">
          <div class="card-top">
            <span class="card-name">{name}</span>
            <span class="card-status {status_class}">{status_text}</span>
          </div>
          <div class="card-stats">
            <div>HP <span class="stat-val">{health}</span></div>
            <div>STR <span class="stat-val">{strength}</span></div>
            <div>DEF <span class="stat-val">{defense}</span></div>
            <div>INT <span class="stat-val">{intelligence}</span></div>
          </div>
          <div class="card-bands">
            <span style="color:{hunger_c}">{hunger}</span>
            <span style="color:{thirst_c}">{thirst}</span>
            <span style="color:{stamina_c}">{stamina}</span>
          </div>
        </div>"#,
        name = html_escape(&tribute.name),
        strength = tribute.attributes.strength,
        defense = tribute.attributes.defense,
        intelligence = tribute.attributes.intelligence,
    )
}

pub fn render_area_card(area: &game::areas::AreaDetails) -> String {
    let item_count = area.items.len();
    let event_count = area.events.len();

    format!(
        r#"<div class="area-card">
          <div class="card-top">
            <span class="card-name">{name}</span>
            <span class="card-status open">OPEN</span>
          </div>
          <div class="card-items">
            {items}
          </div>
          <div class="card-events">
            {events} events
          </div>
        </div>"#,
        name = html_escape(&area.name),
        items = if item_count > 0 {
            format!("{} items", item_count)
        } else {
            "No items".to_string()
        },
        events = event_count,
    )
}

pub fn render_tribute_detail(tribute: &game::tributes::Tribute, _game_id: &str) -> String {
    let is_alive = tribute.is_alive();
    let health = tribute.attributes.health;
    let status_class = if is_alive { "alive" } else { "dead" };
    let status_text = if is_alive { "ALIVE" } else { "DEAD" };
    let hunger = hunger_label(tribute.hunger);
    let hunger_c = hunger_color(tribute.hunger);
    let thirst = thirst_label(tribute.thirst);
    let thirst_c = thirst_color(tribute.thirst);
    let stamina = stamina_label(tribute.stamina, tribute.max_stamina);
    let stamina_c = stamina_color(tribute.stamina, tribute.max_stamina);

    let mut items_html = String::new();
    for item in &tribute.items {
        items_html.push_str(&format!(
            "<span class=\"item-tag\">{}</span>",
            html_escape(&item.name)
        ));
    }
    if items_html.is_empty() {
        items_html = "<span style=\"color:var(--muted);font-size:var(--fs-xs);\">No items</span>"
            .to_string();
    }

    let mut afflictions_html = String::new();
    for kind in tribute.afflictions.keys() {
        afflictions_html.push_str(&format!(
            "<span class=\"affliction-badge severity-moderate\">{}</span>",
            html_escape(&format!("{:?}", kind))
        ));
    }

    format!(
        r#"<div class="detail-header">
          <div>
            <h1>{name}</h1>
            <span class="card-status {status_class}">{status_text}</span>
            District {district}
          </div>
        </div>
        <div class="card-stats">
          <div>HP <span class="stat-val">{health}</span></div>
          <div>STR <span class="stat-val">{strength}</span></div>
          <div>DEF <span class="stat-val">{defense}</span></div>
          <div>INT <span class="stat-val">{intelligence}</span></div>
        </div>
        <div class="card-bands">
          <span style="color:{hunger_c}">{hunger}</span>
          <span style="color:{thirst_c}">{thirst}</span>
          <span style="color:{stamina_c}">{stamina}</span>
        </div>
        <div class="card-items">{items}</div>
        <div class="card-afflictions">{afflictions}</div>"#,
        name = html_escape(&tribute.name),
        district = tribute.district,
        strength = tribute.attributes.strength,
        defense = tribute.attributes.defense,
        intelligence = tribute.attributes.intelligence,
        items = items_html,
        afflictions = afflictions_html,
    )
}

const ALLIANCE_COLORS: &[&str] = &[
    "var(--broad-accent)",
    "var(--info)",
    "var(--gold)",
    "var(--purple)",
    "var(--waiting)",
    "var(--running)",
    "var(--danger)",
];

pub struct AllianceGroup {
    pub name: String,
    pub color: &'static str,
    pub tributes: Vec<usize>,
}

pub fn build_alliance_groups(tributes: &[&game::tributes::Tribute]) -> Vec<AllianceGroup> {
    let n = tributes.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], x: usize) -> usize {
        if parent[x] != x {
            parent[x] = find(parent, parent[x]);
        }
        parent[x]
    }

    fn union(parent: &mut [usize], a: usize, b: usize) {
        let ra = find(parent, a);
        let rb = find(parent, b);
        if ra != rb {
            parent[ra] = rb;
        }
    }

    let id_to_idx: std::collections::HashMap<String, usize> = tributes
        .iter()
        .enumerate()
        .map(|(i, t)| (t.id.to_string(), i))
        .collect();

    for (i, tribute) in tributes.iter().enumerate() {
        for ally_id in &tribute.allies {
            if let Some(&j) = id_to_idx.get(&ally_id.to_string()) {
                union(&mut parent, i, j);
            }
        }
    }

    let mut groups: std::collections::HashMap<usize, Vec<usize>> = std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(i);
    }

    let mut alliance_groups: Vec<AllianceGroup> = groups
        .into_iter()
        .enumerate()
        .map(|(color_idx, (_, indices))| {
            let color = ALLIANCE_COLORS[color_idx % ALLIANCE_COLORS.len()];
            let name = if indices.len() == 1 {
                "SOLO".to_string()
            } else {
                format!("ALLIANCE {}", color_idx + 1)
            };
            AllianceGroup {
                name,
                color,
                tributes: indices,
            }
        })
        .collect();

    alliance_groups.sort_by(|a, b| {
        let a_alive = a
            .tributes
            .iter()
            .filter(|&&i| tributes[i].is_alive())
            .count();
        let b_alive = b
            .tributes
            .iter()
            .filter(|&&i| tributes[i].is_alive())
            .count();
        b_alive.cmp(&a_alive)
    });

    alliance_groups
}

pub fn render_alliance_group(
    group: &AllianceGroup,
    tributes: &[&game::tributes::Tribute],
) -> String {
    let alive_count = group
        .tributes
        .iter()
        .filter(|&&i| tributes[i].is_alive())
        .count();

    let mut rows = String::new();
    for &idx in &group.tributes {
        rows.push_str(&render_tribute_row(tributes[idx]));
    }

    format!(
        r#"<div class="alliance-group">
          <div class="alliance-header" style="border-left-color:{color};">
            <span class="alliance-name">{name}</span>
            <span class="alliance-count">{alive_count}/{total} alive</span>
          </div>
          {rows}
        </div>"#,
        color = group.color,
        name = group.name,
        total = group.tributes.len(),
    )
}

fn terrain_color(terrain: &game::terrain::BaseTerrain) -> &'static str {
    use game::terrain::BaseTerrain::*;
    match terrain {
        Forest => "#2d5a27",
        Desert => "#c4a35a",
        Tundra => "#8fa8b8",
        Wetlands => "#3a7a6e",
        Mountains => "#6b7280",
        UrbanRuins => "#78716c",
        Jungle => "#1a6b3c",
        Grasslands => "#6b8e3a",
        Badlands => "#8b6914",
        Highlands => "#5b7553",
        Geothermal => "#b45309",
        Clearing => "#4a6741",
    }
}

const HEX_SIZE: f64 = 52.0;
const HEX_H: f64 = 104.0;
const HEX_W: f64 = 90.0;
const CENTER_X: f64 = 160.0;
const CENTER_Y: f64 = 150.0;

fn hex_corners(cx: f64, cy: f64) -> [(f64, f64); 6] {
    let mut corners = [(0.0, 0.0); 6];
    for (i, corner) in corners.iter_mut().enumerate() {
        let angle = std::f64::consts::PI / 180.0 * (60.0 * i as f64 - 30.0);
        *corner = (cx + HEX_SIZE * angle.cos(), cy + HEX_SIZE * angle.sin());
    }
    corners
}

fn hex_center(area_idx: usize) -> (f64, f64) {
    match area_idx {
        0 => (CENTER_X, CENTER_Y),
        1 => (CENTER_X + HEX_W, CENTER_Y),
        2 => (CENTER_X + HEX_W * 0.5, CENTER_Y - HEX_H * 0.75),
        3 => (CENTER_X - HEX_W * 0.5, CENTER_Y - HEX_H * 0.75),
        4 => (CENTER_X - HEX_W, CENTER_Y),
        5 => (CENTER_X - HEX_W * 0.5, CENTER_Y + HEX_H * 0.75),
        6 => (CENTER_X + HEX_W * 0.5, CENTER_Y + HEX_H * 0.75),
        _ => (CENTER_X, CENTER_Y),
    }
}

pub fn render_hex_map(
    areas: &[game::areas::AreaDetails],
    tributes: &[&game::tributes::Tribute],
) -> String {
    let area_order = [
        game::areas::Area::Cornucopia,
        game::areas::Area::Sector1,
        game::areas::Area::Sector2,
        game::areas::Area::Sector3,
        game::areas::Area::Sector4,
        game::areas::Area::Sector5,
        game::areas::Area::Sector6,
    ];

    let area_map: std::collections::HashMap<game::areas::Area, &game::areas::AreaDetails> = areas
        .iter()
        .filter_map(|a| a.area.map(|area| (area, a)))
        .collect();

    let mut hexes = String::new();

    for (idx, area_type) in area_order.iter().enumerate() {
        let (cx, cy) = hex_center(idx);
        let corners = hex_corners(cx, cy);
        let points: String = corners
            .iter()
            .map(|(x, y)| format!("{x:.1},{y:.1}"))
            .collect::<Vec<_>>()
            .join(" ");

        let terrain = area_map
            .get(area_type)
            .map(|a| a.terrain.base)
            .unwrap_or(game::terrain::BaseTerrain::Clearing);
        let fill = terrain_color(&terrain);

        let terrain_label = format!("{:?}", terrain).to_uppercase();
        let area_name = area_type.to_string();

        let tribute_count = tributes
            .iter()
            .filter(|t| t.is_alive() && *area_type == t.area)
            .count();

        hexes.push_str(&format!(
            r#"<polygon points="{points}" fill="{fill}" stroke="var(--broad-border-strong)" stroke-width="2" opacity="0.85"/>
            <text x="{cx}" y="{cy:.1}" text-anchor="middle" dominant-baseline="middle" fill="rgba(255,255,255,0.9)" font-size="9" font-family="var(--font-condensed)" font-weight="600" letter-spacing="1">{terrain_label}</text>
            <text x="{cx}" y="{cy:.1}" text-anchor="middle" dominant-baseline="middle" fill="rgba(255,255,255,0.5)" font-size="7" font-family="var(--font-condensed)" dy="12">{area_name}</text>"#,
        ));

        // Tribute dots in this hex
        let in_hex: Vec<_> = tributes
            .iter()
            .filter(|t| t.is_alive() && *area_type == t.area)
            .enumerate()
            .map(|(i, t)| {
                let angle = std::f64::consts::PI * 2.0 * i as f64 / tribute_count.max(1) as f64;
                let r = if tribute_count <= 1 { 0.0 } else { 16.0 };
                (t, cx + r * angle.cos(), cy + r * angle.sin())
            })
            .collect();

        for (tribute, dx, dy) in in_hex {
            let dot_color = if tribute.is_alive() {
                "var(--broad-accent)"
            } else {
                "var(--broad-fg-muted)"
            };
            let r = if tribute.is_alive() { 4.0 } else { 2.5 };
            hexes.push_str(&format!(
                r#"<circle cx="{dx:.1}" cy="{dy:.1}" r="{r}" fill="{dot_color}" stroke="var(--broad-bg)" stroke-width="1"/>"#,
            ));
        }
    }

    let view_w = CENTER_X * 2.0 + HEX_W + 20.0;
    let view_h = CENTER_Y * 2.0 + HEX_H + 20.0;

    format!(
        r#"<svg viewBox="0 0 {view_w:.0} {view_h:.0}" xmlns="http://www.w3.org/2000/svg" class="hex-map">
          {hexes}
        </svg>"#,
    )
}

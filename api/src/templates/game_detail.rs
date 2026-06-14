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
        r#"<div class="event-card {archetype}" style="border-left-color:{color};">
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
        r#"<div class="event-card commentary">
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
        r#"<div class="tribute-card{dead_class}">
          <div class="tribute-avatar" style="border-color:{avatar_color};color:{avatar_color}">{initial}</div>
          <div class="tribute-info">
            <div class="tribute-name">{name}</div>
            <div class="tribute-meta">
              <div class="health-bar">
                <div class="health-fill {health_class}" style="width:{health}%;"></div>
              </div>
            </div>
          </div>
          <div class="tribute-stats">
            <span class="tribute-status {status_class}">{status_text}</span>
          </div>
        </div>"#,
        name = html_escape(&tribute.name)
    )
}

pub fn render_tribute_card(tribute: &game::tributes::Tribute) -> String {
    let is_alive = tribute.is_alive();
    let health = tribute.attributes.health;
    let _health_class = if health > 60 { "high" } else if health > 20 { "mid" } else if health > 0 { "low" } else { "empty" };
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
        items = if item_count > 0 { format!("{} items", item_count) } else { "No items".to_string() },
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
        items_html.push_str(&format!("<span class=\"item-tag\">{}</span>", html_escape(&item.name)));
    }
    if items_html.is_empty() {
        items_html = "<span style=\"color:var(--muted);font-size:var(--fs-xs);\">No items</span>".to_string();
    }

    let mut afflictions_html = String::new();
    for (kind, _) in &tribute.afflictions {
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

use std::collections::HashMap;
use tera::{Error, Value};

pub fn icon_filter(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let name = value.as_str().unwrap_or("");
    Ok(Value::String(format!(
        "<svg class=\"icon\"><use href=\"#icon_ui_{name}\"/></svg>"
    )))
}

pub fn narrative_icon_filter(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let name = value.as_str().unwrap_or("");
    Ok(Value::String(format!(
        "<svg class=\"icon\"><use href=\"#icon_narrative_{name}\"/></svg>"
    )))
}

pub fn status_color(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let status = value.as_str().unwrap_or("");
    let color = match status {
        "in_progress" => "var(--running)",
        "not_started" => "var(--waiting)",
        "finished" => "var(--finished)",
        _ => "var(--muted)",
    };
    Ok(Value::String(color.to_string()))
}

pub fn hunger_label(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let v = value.as_u64().unwrap_or(0) as u8;
    let label = match v {
        0 => "Sated",
        1 => "Hungry",
        2 => "Starving",
        _ => "Unknown",
    };
    Ok(Value::String(label.to_string()))
}

pub fn hunger_color(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let v = value.as_u64().unwrap_or(0) as u8;
    let color = match v {
        0 => "var(--running)",
        1 => "var(--waiting)",
        _ => "var(--danger)",
    };
    Ok(Value::String(color.to_string()))
}

pub fn thirst_label(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let v = value.as_u64().unwrap_or(0) as u8;
    let label = match v {
        0 => "Hydrated",
        1 => "Thirsty",
        2 => "Dehydrated",
        _ => "Unknown",
    };
    Ok(Value::String(label.to_string()))
}

pub fn thirst_color(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let v = value.as_u64().unwrap_or(0) as u8;
    let color = match v {
        0 => "var(--running)",
        1 => "var(--waiting)",
        _ => "var(--danger)",
    };
    Ok(Value::String(color.to_string()))
}

pub fn stamina_label(value: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
    let stamina = value.as_u64().unwrap_or(0) as u32;
    let max = args.get("max").and_then(|a| a.as_u64()).unwrap_or(100) as u32;
    let pct = if max > 0 { stamina * 100 / max } else { 0 };
    let label = match pct {
        0..=25 => "Exhausted",
        26..=50 => "Winded",
        _ => "Fresh",
    };
    Ok(Value::String(label.to_string()))
}

pub fn stamina_color(value: &Value, args: &HashMap<String, Value>) -> Result<Value, Error> {
    let stamina = value.as_u64().unwrap_or(0) as u32;
    let max = args.get("max").and_then(|a| a.as_u64()).unwrap_or(100) as u32;
    let pct = if max > 0 { stamina * 100 / max } else { 0 };
    let color = match pct {
        0..=25 => "var(--danger)",
        26..=50 => "var(--waiting)",
        _ => "var(--running)",
    };
    Ok(Value::String(color.to_string()))
}

pub fn message_kind(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let kind = value.as_str().unwrap_or("");
    let archetype = match kind {
        "death" | "wound" | "attack" | "combat" => "action",
        "alliance_formed" | "alliance_proposed" | "alliance_dissolved" | "betrayal"
        | "trust_shock_break" => "action",
        "sponsor_gift" => "commentary",
        "movement" | "hidden" | "area_closed" => "action",
        "area_event" => "event",
        "item_found" | "item_used" | "item_dropped" => "action",
        _ => "action",
    };
    Ok(Value::String(archetype.to_string()))
}

pub fn archetype_label(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let archetype = value.as_str().unwrap_or("");
    let label = match archetype {
        "action" => "ACTION",
        "death" => "DEATHS",
        "event" => "EVENTS",
        "commentary" => "COMMS",
        _ => "OTHER",
    };
    Ok(Value::String(label.to_string()))
}

pub fn kind_color(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let kind = value.as_str().unwrap_or("");
    let color = match kind {
        "death" => "var(--danger)",
        "combat" => "var(--waiting)",
        "alliance" | "betrayal" => "var(--info)",
        "movement" => "var(--accent)",
        "item" => "var(--gold)",
        "hazard" | "event" => "var(--purple)",
        _ => "var(--muted)",
    };
    Ok(Value::String(color.to_string()))
}

pub fn format_words(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let s = value.as_str().unwrap_or("");
    let formatted = s
        .replace('_', " ")
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Value::String(formatted))
}

pub fn upper(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_uppercase()))
}

pub fn lower(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let s = value.as_str().unwrap_or("");
    Ok(Value::String(s.to_lowercase()))
}

pub fn phase_label(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let phase = value.as_str().unwrap_or("");
    let label = match phase {
        "dawn" => "DAWN",
        "day" => "DAY",
        "dusk" => "DUSK",
        "night" => "NIGHT",
        _ => "STAGING",
    };
    Ok(Value::String(label.to_string()))
}

pub fn phase_class(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    let phase = value.as_str().unwrap_or("");
    let class = match phase {
        "dawn" => "phase-dawn",
        "day" => "phase-day",
        "dusk" => "phase-dusk",
        "night" => "phase-night",
        _ => "phase-day",
    };
    Ok(Value::String(class.to_string()))
}

pub fn json(value: &Value, _: &HashMap<String, Value>) -> Result<Value, Error> {
    Ok(Value::String(
        serde_json::to_string(value).unwrap_or_default(),
    ))
}

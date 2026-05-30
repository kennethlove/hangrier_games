//! Narrative descriptor mappings.
//!
//! Raw game values (damage numbers, HP percentages, hit rolls) are mapped
//! to narrative descriptors so the LLM prompt never exposes raw arithmetic.
//! Every function returns a `&'static str` that can be interpolated into
//! `EventLine::prose` or structured data.

// ---------------------------------------------------------------------------
// Damage dealt
// ---------------------------------------------------------------------------

/// Describe how hard a hit landed, based on raw damage dealt.
///
/// | Range  | Descriptor  |
/// |--------|-------------|
/// | 0      | missed      |
/// | 1-4    | glancing    |
/// | 5-9    | solid hit   |
/// | 10-14  | devastating |
/// | 15+    | crushing    |
pub fn describe_damage(damage: u32) -> &'static str {
    match damage {
        0 => "missed",
        1..=4 => "glancing",
        5..=9 => "solid",
        10..=14 => "devastating",
        _ => "crushing",
    }
}

// ---------------------------------------------------------------------------
// Tribute injury level (from current HP %)
// ---------------------------------------------------------------------------

/// Describe a tribute's overall injury level based on remaining HP percentage.
///
/// | HP %    | Descriptor   |
/// |---------|--------------|
/// | 100%    | unharmed     |
/// | 75-99%  | scraped up   |
/// | 50-74%  | wounded      |
/// | 25-49%  | badly wounded|
/// | 1-24%   | near death   |
/// | 0%      | deceased     |
pub fn describe_injury(hp_pct: f64) -> &'static str {
    if hp_pct <= 0.0 {
        "deceased"
    } else if hp_pct < 25.0 {
        "near death"
    } else if hp_pct < 50.0 {
        "badly wounded"
    } else if hp_pct < 75.0 {
        "wounded"
    } else if hp_pct < 100.0 {
        "scraped up"
    } else {
        "unharmed"
    }
}

// ---------------------------------------------------------------------------
// Hit quality (attack roll relative to defense)
// ---------------------------------------------------------------------------

/// Describe how cleanly a blow landed, from the attack-roll perspective.
///
/// | Hit margin | Descriptor      |
/// |------------|-----------------|
/// | miss by 5+ | easily dodged   |
/// | miss 1-4   | just barely dodged |
/// | hit by 1-4 | just connected  |
/// | hit by 5+  | clean hit       |
pub fn describe_hit_quality(hit_margin: i32) -> &'static str {
    if hit_margin < -4 {
        "easily dodged"
    } else if hit_margin < 0 {
        "just barely dodged"
    } else if hit_margin < 5 {
        "just connected"
    } else {
        "clean hit"
    }
}

// ---------------------------------------------------------------------------
// Activity level for an area
// ---------------------------------------------------------------------------

/// Describe how active an area is, based on the count of notable events there.
pub fn describe_area_activity(event_count: u32) -> &'static str {
    match event_count {
        0 => "quiet",
        1..=3 => "active",
        _ => "hot",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn damage_descriptions() {
        assert_eq!(describe_damage(0), "missed");
        assert_eq!(describe_damage(1), "glancing");
        assert_eq!(describe_damage(4), "glancing");
        assert_eq!(describe_damage(5), "solid");
        assert_eq!(describe_damage(9), "solid");
        assert_eq!(describe_damage(10), "devastating");
        assert_eq!(describe_damage(14), "devastating");
        assert_eq!(describe_damage(15), "crushing");
        assert_eq!(describe_damage(99), "crushing");
    }

    #[test]
    fn injury_descriptions() {
        assert_eq!(describe_injury(100.0), "unharmed");
        assert_eq!(describe_injury(99.9), "scraped up");
        assert_eq!(describe_injury(75.0), "scraped up");
        assert_eq!(describe_injury(74.9), "wounded");
        assert_eq!(describe_injury(50.0), "wounded");
        assert_eq!(describe_injury(49.9), "badly wounded");
        assert_eq!(describe_injury(25.0), "badly wounded");
        assert_eq!(describe_injury(24.9), "near death");
        assert_eq!(describe_injury(1.0), "near death");
        assert_eq!(describe_injury(0.0), "deceased");
    }

    #[test]
    fn hit_quality_descriptions() {
        assert_eq!(describe_hit_quality(-10), "easily dodged");
        assert_eq!(describe_hit_quality(-5), "easily dodged");
        assert_eq!(describe_hit_quality(-4), "just barely dodged");
        assert_eq!(describe_hit_quality(-1), "just barely dodged");
        assert_eq!(describe_hit_quality(0), "just connected");
        assert_eq!(describe_hit_quality(4), "just connected");
        assert_eq!(describe_hit_quality(5), "clean hit");
        assert_eq!(describe_hit_quality(20), "clean hit");
    }

    #[test]
    fn area_activity_descriptions() {
        assert_eq!(describe_area_activity(0), "quiet");
        assert_eq!(describe_area_activity(1), "active");
        assert_eq!(describe_area_activity(3), "active");
        assert_eq!(describe_area_activity(4), "hot");
        assert_eq!(describe_area_activity(99), "hot");
    }
}

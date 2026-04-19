use serde::{Deserialize, Serialize};

/// Configuration for game constants and tuning parameters.
/// Centralizes magic numbers to enable runtime configuration and difficulty modes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GameConfig {
    // Game lifecycle constants (from games.rs)
    /// Tribute count threshold for area constriction
    pub low_tribute_threshold: u32,
    /// Number of weapons spawned during feast
    pub feast_weapon_count: u32,
    /// Number of shields spawned during feast
    pub feast_shield_count: u32,
    /// Number of consumables spawned during feast
    pub feast_consumable_count: u32,
    /// Probability of day events occurring (1.0 = 100%, 0.25 = 25%)
    pub day_event_frequency: f64,
    /// Probability of night events occurring (1.0 = 100%, 0.125 = 12.5%)
    pub night_event_frequency: f64,
    /// Enable instant death outcomes for catastrophic events
    pub instant_death_enabled: bool,
    /// Global multiplier for event severity (1.0 = normal, 2.0 = double damage)
    pub catastrophic_severity_multiplier: f64,

    // Tribute AI decision thresholds (from tributes/brains.rs)
    /// Enemy count threshold for "few enemies" AI decisions
    pub low_enemy_limit: u32,
    /// Health threshold for low health AI decisions
    pub low_health_limit: u32,
    /// Health threshold for mid health AI decisions
    pub mid_health_limit: u32,
    /// Extreme low sanity threshold for desperate actions
    pub extreme_low_sanity_limit: u32,
    /// Low sanity threshold for impulsive actions
    pub low_sanity_limit: u32,
    /// Mid sanity threshold for cautious actions
    pub mid_sanity_limit: u32,
    /// Low movement threshold for exhaustion checks
    pub low_movement_limit: u32,
    /// High intelligence threshold for tactical decisions
    pub high_intelligence_limit: u32,
    /// Low intelligence threshold for reckless decisions
    pub low_intelligence_limit: u32,

    // Tribute lifecycle constants (from tributes/mod.rs)
    /// Sanity level at which tributes may attempt suicide
    pub sanity_break_level: u32,
    /// Loyalty percentage below which tributes may betray allies
    pub loyalty_break_level: f64,

    // Attribute maximums (from tributes/mod.rs)
    pub max_health: u32,
    pub max_sanity: u32,
    pub max_movement: u32,
    pub max_strength: u32,
    pub max_defense: u32,
    pub max_bravery: u32,
    pub max_loyalty: u32,
    pub max_intelligence: u32,
    pub max_persuasion: u32,
    pub max_luck: u32,
}

impl Default for GameConfig {
    /// Returns default configuration matching the original hardcoded values.
    fn default() -> Self {
        Self {
            // Game lifecycle
            low_tribute_threshold: 8,
            feast_weapon_count: 2,
            feast_shield_count: 2,
            feast_consumable_count: 4,
            day_event_frequency: 1.0 / 4.0,
            night_event_frequency: 1.0 / 8.0,
            instant_death_enabled: true,
            catastrophic_severity_multiplier: 1.0,

            // Tribute AI
            low_enemy_limit: 6,
            low_health_limit: 20,
            mid_health_limit: 40,
            extreme_low_sanity_limit: 10,
            low_sanity_limit: 20,
            mid_sanity_limit: 35,
            low_movement_limit: 10,
            high_intelligence_limit: 35,
            low_intelligence_limit: 80,

            // Tribute lifecycle
            sanity_break_level: 9,
            loyalty_break_level: 0.25,

            // Attribute maximums
            max_health: 100,
            max_sanity: 100,
            max_movement: 100,
            max_strength: 50,
            max_defense: 50,
            max_bravery: 100,
            max_loyalty: 100,
            max_intelligence: 100,
            max_persuasion: 100,
            max_luck: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.low_tribute_threshold, 8);
        assert_eq!(config.feast_weapon_count, 2);
        assert_eq!(config.max_health, 100);
        assert_eq!(config.low_health_limit, 20);
    }

    #[test]
    fn test_config_serialization() {
        let config = GameConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_event_config_defaults() {
        let config = GameConfig::default();
        assert_eq!(config.day_event_frequency, 0.25);
        assert_eq!(config.night_event_frequency, 0.125);
        assert!(config.instant_death_enabled);
        assert_eq!(config.catastrophic_severity_multiplier, 1.0);
    }

    #[test]
    fn test_easy_mode_config() {
        let config = GameConfig {
            instant_death_enabled: false,
            catastrophic_severity_multiplier: 0.5,
            ..GameConfig::default()
        };

        assert!(!config.instant_death_enabled);
        assert_eq!(config.catastrophic_severity_multiplier, 0.5);
    }

    #[test]
    fn test_hard_mode_config() {
        let config = GameConfig {
            instant_death_enabled: true,
            catastrophic_severity_multiplier: 2.0,
            day_event_frequency: 0.5,
            ..GameConfig::default()
        };

        assert_eq!(config.catastrophic_severity_multiplier, 2.0);
        assert_eq!(config.day_event_frequency, 0.5);
    }
}

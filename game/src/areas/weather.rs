use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Weather {
    #[default]
    Clear,
    HeavyRain,
    Heatwave,
    Blizzard,
}

/// Stub producer. Always returns `Weather::Clear` until the full weather
/// system (see `2026-05-02-weather-system-design.md`) replaces this.
///
/// Consumers must call this (not hardcode `Weather::Clear`) so the future
/// weather implementation needs only a producer-side change.
pub fn current_weather() -> Weather {
    Weather::Clear
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weather_default_is_clear() {
        assert_eq!(Weather::default(), Weather::Clear);
    }

    #[test]
    fn current_weather_stub_returns_clear() {
        assert_eq!(current_weather(), Weather::Clear);
    }
}

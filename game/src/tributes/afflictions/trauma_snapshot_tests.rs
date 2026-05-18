#[cfg(test)]
mod tests {
    use crate::tributes::Tribute;
    use shared::afflictions::*;

    #[test]
    fn canonical_trauma_with_three_sources_and_observers() {
        let mut t = Tribute {
            identifier: "tributes:canonical".into(),
            game_day: Some(10),
            ..Default::default()
        };

        // Acquire trauma.
        t.try_acquire_trauma(
            TraumaSource::WitnessedAllyDeath {
                ally: "tributes:glimmer".into(),
                cause: Some(DeathCause::Fire),
            },
            Severity::Mild,
        );

        // Reinforce twice with different sources, ending at Severe via floor bumps.
        t.game_day = Some(12);
        t.try_acquire_trauma(
            TraumaSource::Betrayal {
                by: "tributes:marvel".into(),
            },
            Severity::Moderate,
        );

        t.game_day = Some(15);
        t.try_acquire_trauma(
            TraumaSource::MassCasualty {
                cause_class: CauseClass::Combat,
                deaths_this_cycle: 5,
            },
            Severity::Severe,
        );

        // Manually populate observer state to lock the wire shape (PR3 will
        // populate this via flashback firings; PR1 captures the schema only).
        let trauma = t
            .afflictions
            .get_mut(&(AfflictionKind::Trauma, None))
            .unwrap();
        let meta = trauma.trauma_metadata.as_mut().unwrap();
        meta.observed_by.insert("tributes:cato".into());
        meta.observer_seen_cycle.insert("tributes:cato".into(), 14);
        meta.cycles_since_last_event = 0;

        let json = serde_json::to_string_pretty(&trauma).unwrap();
        insta::assert_snapshot!("canonical_trauma", json);
    }
}

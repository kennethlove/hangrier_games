//! Stamina-as-combat-resource integration tests (PR1 backend).
//! See docs/superpowers/specs/2026-05-03-stamina-combat-resource-design.md.

use game::games::Game;
use game::tributes::Tribute;
use shared::messages::{MessagePayload, StaminaBand};

#[test]
fn per_phase_loop_emits_stamina_band_changed_when_band_crosses() {
    // Tribute starts at 19/100 stamina (Exhausted). After one phase of idle
    // recovery (+5) they reach 24/100 (Winded). We expect a
    // StaminaBandChanged Exhausted -> Winded message in the game log.
    let mut g = Game::default();
    let mut t = Tribute::new("Tester".to_string(), None, None);
    t.stamina = 19;
    t.max_stamina = 100;
    let id = t.identifier.clone();
    g.tributes.push(t);

    let _ = g.run_phase(shared::messages::Phase::Day);

    let crossed = g.messages.iter().any(|m| {
        matches!(&m.payload,
            MessagePayload::StaminaBandChanged { tribute, from, to }
                if tribute.identifier == id
                    && *from == StaminaBand::Exhausted
                    && *to == StaminaBand::Winded
        )
    });
    assert!(
        crossed,
        "expected Exhausted -> Winded StaminaBandChanged event for {id}"
    );
}

#[test]
fn fresh_tribute_emits_no_band_change_when_recovery_keeps_band() {
    // Tribute at 100/100 stamina stays Fresh after a phase (capped at max).
    // Verify NO StaminaBandChanged event fires for them.
    let mut g = Game::default();
    let mut t = Tribute::new("Steady".to_string(), None, None);
    t.stamina = 100;
    t.max_stamina = 100;
    let id = t.identifier.clone();
    g.tributes.push(t);

    let _ = g.run_phase(shared::messages::Phase::Day);

    let crossed = g.messages.iter().any(|m| {
        matches!(&m.payload, MessagePayload::StaminaBandChanged { tribute, .. } if tribute.identifier == id)
    });
    assert!(
        !crossed,
        "Fresh tribute should not emit a StaminaBandChanged event"
    );
}

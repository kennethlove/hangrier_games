pub mod display;
pub mod types;
pub use types::GameEvent;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{Attribute, Item, ItemRarity, ItemType};
    use crate::output::GameOutput;
    use crate::threats::animals::Animal;
    use uuid::Uuid;

    /// Stable UUIDs so test failures are easy to reason about.
    fn uid_a() -> Uuid {
        Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap()
    }
    fn uid_b() -> Uuid {
        Uuid::parse_str("22222222-2222-2222-2222-222222222222").unwrap()
    }

    fn sample_item() -> Item {
        Item {
            identifier: "33333333-3333-3333-3333-333333333333".to_string().into(),
            name: "elixir".to_string(),
            item_type: ItemType::Consumable,
            rarity: ItemRarity::Common,
            current_durability: 3,
            max_durability: 5,
            attribute: Attribute::Health,
            effect: 7,
        }
    }

    /// Single source of truth for the parity table. Each row pairs a
    /// constructed `GameEvent` with a `GameOutput` carrying the same data;
    /// the rendered strings must be byte-identical.
    fn parity_table() -> Vec<(GameEvent, GameOutput<'static>)> {
        let item = sample_item();
        // SAFETY: `Item` is owned, but `GameOutput` borrows. We leak the
        // sample item once for the test table so its references are 'static.
        // This is test-only code; the leak is bounded and intentional.
        let item_ref: &'static Item = Box::leak(Box::new(item.clone()));

        vec![
            (
                GameEvent::GameDayStart { day_number: 4 },
                GameOutput::GameDayStart(4),
            ),
            (
                GameEvent::GameDayEnd { day_number: 4 },
                GameOutput::GameDayEnd(4),
            ),
            (GameEvent::FirstDayStart, GameOutput::FirstDayStart),
            (GameEvent::FeastDayStart, GameOutput::FeastDayStart),
            (
                GameEvent::TributesLeft { tribute_count: 12 },
                GameOutput::TributesLeft(12),
            ),
            (
                GameEvent::GameNightStart { day_number: 2 },
                GameOutput::GameNightStart(2),
            ),
            (
                GameEvent::GameNightEnd { day_number: 2 },
                GameOutput::GameNightEnd(2),
            ),
            (
                GameEvent::DailyDeathAnnouncement { death_count: 3 },
                GameOutput::DailyDeathAnnouncement(3),
            ),
            (
                GameEvent::DeathAnnouncement {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::DeathAnnouncement("Alice"),
            ),
            (GameEvent::NoOneWins, GameOutput::NoOneWins),
            (
                GameEvent::TributeWins {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeWins("Alice"),
            ),
            (
                GameEvent::TributeRest {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeRest("Alice"),
            ),
            (
                GameEvent::TributeLongRest {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeLongRest("Alice"),
            ),
            (
                GameEvent::TributeHide {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeHide("Alice"),
            ),
            (
                GameEvent::TributeTravel {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    from_area: "Cornucopia".into(),
                    to_area: "North".into(),
                },
                GameOutput::TributeTravel("Alice", "Cornucopia", "North"),
            ),
            (
                GameEvent::TributeTakeItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item_name: "elixir".into(),
                },
                GameOutput::TributeTakeItem("Alice", "elixir"),
            ),
            (
                GameEvent::TributeCannotUseItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item_name: "elixir".into(),
                },
                GameOutput::TributeCannotUseItem("Alice", "elixir"),
            ),
            (
                GameEvent::TributeUseItem {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item: item.clone(),
                },
                GameOutput::TributeUseItem("Alice", item_ref),
            ),
            (
                GameEvent::TributeTravelTooTired {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelTooTired("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelExhausted {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelExhausted("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelAlreadyThere {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelAlreadyThere("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelFollow {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelFollow("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelStay {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelStay("Alice", "Forest"),
            ),
            (
                GameEvent::TributeTravelNoOptions {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area: "Forest".into(),
                },
                GameOutput::TributeTravelNoOptions("Alice", "Forest"),
            ),
            (
                GameEvent::TributeBleeds {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBleeds("Alice"),
            ),
            (
                GameEvent::TributeSick {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSick("Alice"),
            ),
            (
                GameEvent::TributeElectrocuted {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeElectrocuted("Alice"),
            ),
            (
                GameEvent::TributeFrozen {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeFrozen("Alice"),
            ),
            (
                GameEvent::TributeOverheated {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeOverheated("Alice"),
            ),
            (
                GameEvent::TributeDehydrated {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDehydrated("Alice"),
            ),
            (
                GameEvent::TributeStarving {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeStarving("Alice"),
            ),
            (
                GameEvent::TributePoisoned {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributePoisoned("Alice"),
            ),
            (
                GameEvent::TributeMauled {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    animal_count: 3,
                    animal: Animal::Wolf,
                    damage: 12,
                },
                GameOutput::TributeMauled("Alice", 3, "Wolf", 12),
            ),
            (
                GameEvent::TributeBurned {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeBurned("Alice"),
            ),
            (
                GameEvent::TributeHorrified {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    sanity_damage: 5,
                },
                GameOutput::TributeHorrified("Alice", 5),
            ),
            (
                GameEvent::TributeSuffer {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSuffer("Alice"),
            ),
            (
                GameEvent::TributeSelfHarm {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSelfHarm("Alice"),
            ),
            (
                GameEvent::TributeSuicide {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeSuicide("Alice"),
            ),
            (
                GameEvent::TributeAttackWin {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWin("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackWinExtra {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWinExtra("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackWound {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackWound("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackLose {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackLose("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackLoseExtra {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackLoseExtra("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackMiss {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackMiss("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackDied {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackDied("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackSuccessKill {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackSuccessKill("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAttackHidden {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeAttackHidden("Alice", "Bob"),
            ),
            (
                GameEvent::TributeCriticalHit {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeCriticalHit("Alice", "Bob"),
            ),
            (
                GameEvent::TributeCriticalFumble {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeCriticalFumble("Alice"),
            ),
            (
                GameEvent::TributePerfectBlock {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributePerfectBlock("Alice", "Bob"),
            ),
            (
                GameEvent::TributeDiesFromStatus {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    status: "poison".into(),
                },
                GameOutput::TributeDiesFromStatus("Alice", "poison"),
            ),
            (
                GameEvent::TributeDiesFromAreaEvent {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_event: "wildfire".into(),
                },
                GameOutput::TributeDiesFromAreaEvent("Alice", "wildfire"),
            ),
            (
                GameEvent::TributeDiesFromTributeEvent {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    tribute_event: "Bob".into(),
                },
                GameOutput::TributeDiesFromTributeEvent("Alice", "Bob"),
            ),
            (
                GameEvent::TributeAlreadyDead {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeAlreadyDead("Alice"),
            ),
            (
                GameEvent::TributeDead {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDead("Alice"),
            ),
            (
                GameEvent::TributeDeath {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::TributeDeath("Alice"),
            ),
            (
                GameEvent::WeaponBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    weapon_name: "spear".into(),
                },
                GameOutput::WeaponBreak("Alice", "spear"),
            ),
            (
                GameEvent::WeaponWear {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    weapon_name: "spear".into(),
                },
                GameOutput::WeaponWear("Alice", "spear"),
            ),
            (
                GameEvent::ShieldBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    shield_name: "buckler".into(),
                },
                GameOutput::ShieldBreak("Alice", "buckler"),
            ),
            (
                GameEvent::ShieldWear {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    shield_name: "buckler".into(),
                },
                GameOutput::ShieldWear("Alice", "buckler"),
            ),
            (
                GameEvent::SponsorGift {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    item: item.clone(),
                },
                GameOutput::SponsorGift("Alice", item_ref),
            ),
            (
                GameEvent::AreaEvent {
                    area_event: "earthquake".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaEvent("earthquake", "The Forest"),
            ),
            (
                GameEvent::AreaClose {
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaClose("The Forest"),
            ),
            (
                GameEvent::AreaOpen {
                    area_name: "The Forest".into(),
                },
                GameOutput::AreaOpen("The Forest"),
            ),
            (
                GameEvent::TrappedInArea {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::TrappedInArea("Alice", "The Forest"),
            ),
            (
                GameEvent::DiedInArea {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    area_name: "The Forest".into(),
                },
                GameOutput::DiedInArea("Alice", "The Forest"),
            ),
            (
                GameEvent::TributeBetrayal {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeBetrayal("Alice", "Bob"),
            ),
            (
                GameEvent::TributeForcedBetrayal {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                    target_id: uid_b(),
                    target_name: "Bob".into(),
                },
                GameOutput::TributeForcedBetrayal("Alice", "Bob"),
            ),
            (
                GameEvent::NoOneToAttack {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::NoOneToAttack("Alice"),
            ),
            (
                GameEvent::AllAlone {
                    tribute_id: uid_a(),
                    tribute_name: "Alice".into(),
                },
                GameOutput::AllAlone("Alice"),
            ),
            (
                GameEvent::AllianceFormed {
                    tribute_a_id: uid_a(),
                    tribute_a_name: "Alice".into(),
                    tribute_b_id: uid_b(),
                    tribute_b_name: "Bob".into(),
                    factor: "trust".into(),
                },
                GameOutput::AllianceFormed("Alice", "Bob", "trust"),
            ),
            (
                GameEvent::BetrayalTriggered {
                    betrayer_id: uid_a(),
                    betrayer_name: "Cato".into(),
                    victim_id: uid_b(),
                    victim_name: "Glimmer".into(),
                },
                GameOutput::BetrayalTriggered("Cato", "Glimmer"),
            ),
            (
                GameEvent::TrustShockBreak {
                    tribute_id: uid_a(),
                    tribute_name: "Rue".into(),
                },
                GameOutput::TrustShockBreak("Rue"),
            ),
        ]
    }

    #[test]
    fn parity_table_covers_every_variant() {
        // Bumps any time a variant is added without a parity row.
        // 73 = current count of GameEvent variants in types.rs.
        assert_eq!(parity_table().len(), 73);
    }

    #[test]
    fn display_matches_game_output_for_every_variant() {
        for (event, output) in parity_table() {
            assert_eq!(
                event.to_string(),
                output.to_string(),
                "Display mismatch for {:?}",
                event
            );
        }
    }

    // ---------- Serde roundtrip coverage ----------
    // One assertion per data shape: unit, single-field, multi-field,
    // optional-field-via-Item.

    fn roundtrip(event: &GameEvent) {
        let json = serde_json::to_string(event).expect("serialize");
        let parsed: GameEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(*event, parsed, "roundtrip mismatch: {}", json);
    }

    #[test]
    fn serde_roundtrip_unit_variant() {
        roundtrip(&GameEvent::FirstDayStart);
        roundtrip(&GameEvent::FeastDayStart);
        roundtrip(&GameEvent::NoOneWins);
    }

    #[test]
    fn serde_roundtrip_single_primitive_field() {
        roundtrip(&GameEvent::GameDayStart { day_number: 7 });
        roundtrip(&GameEvent::TributesLeft { tribute_count: 11 });
    }

    #[test]
    fn serde_roundtrip_multi_field_with_uuid() {
        roundtrip(&GameEvent::AllianceFormed {
            tribute_a_id: uid_a(),
            tribute_a_name: "Alice".into(),
            tribute_b_id: uid_b(),
            tribute_b_name: "Bob".into(),
            factor: "shared district".into(),
        });
    }

    #[test]
    fn serde_roundtrip_with_nested_item() {
        roundtrip(&GameEvent::SponsorGift {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            item: sample_item(),
        });
        roundtrip(&GameEvent::TributeUseItem {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            item: sample_item(),
        });
    }

    #[test]
    fn serde_roundtrip_with_animal_enum() {
        roundtrip(&GameEvent::TributeMauled {
            tribute_id: uid_a(),
            tribute_name: "Alice".into(),
            animal_count: 4,
            animal: Animal::TrackerJacker,
            damage: 9,
        });
    }
}

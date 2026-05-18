#[cfg(test)]
mod tests {
    use serde::Serialize;
    use shared::afflictions::{Affliction, AfflictionKind, AfflictionSource, BodyPart, Severity};
    use std::collections::BTreeMap;

    #[derive(Serialize)]
    struct SerializableEntry {
        key_kind: String,
        key_body_part: Option<String>,
        severity: String,
        source: String,
    }

    #[test]
    fn test_affliction_map_serialization_snapshot() {
        let mut map: BTreeMap<(AfflictionKind, Option<BodyPart>), Affliction> = BTreeMap::new();

        map.insert(
            (AfflictionKind::Wounded, Some(BodyPart::Arm)),
            Affliction {
                kind: AfflictionKind::Wounded,
                body_part: Some(BodyPart::Arm),
                severity: Severity::Moderate,
                source: AfflictionSource::Combat {
                    attacker_id: "tributes:test".into(),
                },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            },
        );
        map.insert(
            (AfflictionKind::MissingLeg, None),
            Affliction {
                kind: AfflictionKind::MissingLeg,
                body_part: None,
                severity: Severity::Severe,
                source: AfflictionSource::Combat {
                    attacker_id: "tributes:test".into(),
                },
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            },
        );
        map.insert(
            (AfflictionKind::Burned, None),
            Affliction {
                kind: AfflictionKind::Burned,
                body_part: None,
                severity: Severity::Severe,
                source: AfflictionSource::Environmental,
                acquired_cycle: 1,
                last_progressed_cycle: 1,
                trauma_metadata: None,
            },
        );

        let entries: Vec<SerializableEntry> = map
            .iter()
            .map(|((kind, body_part), affl)| SerializableEntry {
                key_kind: kind.to_string(),
                key_body_part: body_part.map(|bp| bp.to_string()),
                severity: affl.severity.to_string(),
                source: match &affl.source {
                    AfflictionSource::Spawn => "spawn",
                    AfflictionSource::Combat { .. } => "combat",
                    AfflictionSource::Environmental => "environmental",
                    AfflictionSource::Cascade { .. } => "cascade",
                    AfflictionSource::Sponsor => "sponsor",
                    AfflictionSource::Gamemaker => "gamemaker",
                }
                .to_string(),
            })
            .collect();

        let json = serde_json::to_string_pretty(&entries).unwrap();
        insta::assert_snapshot!("affliction_map_serialization", json);
    }
}

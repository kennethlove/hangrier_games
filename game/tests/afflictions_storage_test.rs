//! Integration tests for affliction storage and serde round-trip.

use game::tributes::afflictions::AcquireResolution;
use game::tributes::{AfflictionDraft, Tribute};
use shared::afflictions::{AfflictionKind, AfflictionSource, BodyPart, Severity};

/// Test that a full acquisition flow works end-to-end.
#[test]
fn test_full_acquisition_flow() {
    let mut tribute = Tribute::new("Test".to_string(), Some(1), Some("1".to_string()));

    // Acquire first affliction
    let resolution = tribute.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Mild,
        source: AfflictionSource::Combat,
    });
    assert!(matches!(resolution, AcquireResolution::Insert));
    assert_eq!(tribute.afflictions.len(), 1);

    // Upgrade severity
    let resolution = tribute.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Moderate,
        source: AfflictionSource::Combat,
    });
    assert!(matches!(resolution, AcquireResolution::Upgrade(_)));
    assert_eq!(tribute.afflictions.len(), 1); // Still 1, upgraded in place

    // Acquire second affliction on different body part
    let resolution = tribute.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::BrokenBone,
        body_part: Some(BodyPart::Leg),
        severity: Severity::Severe,
        source: AfflictionSource::Combat,
    });
    assert!(matches!(resolution, AcquireResolution::Insert));
    assert_eq!(tribute.afflictions.len(), 2);
}

/// Test that afflictions survive a serde round-trip.
/// Uses value-based serialization since tuple keys cannot be JSON map keys.
#[test]
fn test_affliction_serde_round_trip() {
    let mut tribute = Tribute::new("Test".to_string(), Some(1), Some("1".to_string()));

    tribute.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Wounded,
        body_part: Some(BodyPart::Arm),
        severity: Severity::Moderate,
        source: AfflictionSource::Combat,
    });
    tribute.try_acquire_affliction(AfflictionDraft {
        kind: AfflictionKind::Burned,
        body_part: None,
        severity: Severity::Severe,
        source: AfflictionSource::Environmental,
    });

    assert_eq!(tribute.afflictions.len(), 2);

    // Serialize affliction values as Vec (tuple keys can't be JSON map keys)
    let afflictions_vec: Vec<_> = tribute.afflictions.values().cloned().collect();
    let json = serde_json::to_string(&afflictions_vec).unwrap();

    // Deserialize
    let restored: Vec<shared::afflictions::Affliction> = serde_json::from_str(&json).unwrap();

    // Verify afflictions survived round-trip
    assert_eq!(restored.len(), 2);
    assert!(
        restored
            .iter()
            .any(|a| a.kind == AfflictionKind::Wounded && a.body_part == Some(BodyPart::Arm))
    );
    assert!(
        restored
            .iter()
            .any(|a| a.kind == AfflictionKind::Burned && a.body_part.is_none())
    );
}

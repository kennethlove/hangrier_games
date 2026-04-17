use shared::{EventFrequency, ItemQuantity};

/// Test ItemQuantity enum provides correct base counts.
#[test]
fn test_item_quantity_base_counts() {
    assert_eq!(ItemQuantity::Scarce.base_item_count(), 1);
    assert_eq!(ItemQuantity::Normal.base_item_count(), 3);
    assert_eq!(ItemQuantity::Abundant.base_item_count(), 5);
}

/// Test that Normal is the default ItemQuantity.
#[test]
fn test_item_quantity_default() {
    let default_qty: ItemQuantity = Default::default();
    assert_eq!(default_qty, ItemQuantity::Normal);
    assert_eq!(default_qty.base_item_count(), 3);
}

/// Test EventFrequency enum provides correct probabilities.
#[test]
fn test_event_frequency_probabilities() {
    assert!((EventFrequency::Rare.event_probability() - 0.1).abs() < 0.001);
    assert!((EventFrequency::Normal.event_probability() - 0.25).abs() < 0.001);
    assert!((EventFrequency::Frequent.event_probability() - 0.5).abs() < 0.001);
}

/// Test that Normal is the default EventFrequency.
#[test]
fn test_event_frequency_default() {
    let default_freq: EventFrequency = Default::default();
    assert_eq!(default_freq, EventFrequency::Normal);
    assert!((default_freq.event_probability() - 0.25).abs() < 0.001);
}

/// Test serialization and deserialization of ItemQuantity.
#[test]
fn test_item_quantity_serde() {
    let qty = ItemQuantity::Abundant;
    let json = serde_json::to_string(&qty).unwrap();
    let deserialized: ItemQuantity = serde_json::from_str(&json).unwrap();
    assert_eq!(qty, deserialized);
}

/// Test serialization and deserialization of EventFrequency.
#[test]
fn test_event_frequency_serde() {
    let freq = EventFrequency::Rare;
    let json = serde_json::to_string(&freq).unwrap();
    let deserialized: EventFrequency = serde_json::from_str(&json).unwrap();
    assert_eq!(freq, deserialized);
}

/// Test that all ItemQuantity variants can be serialized/deserialized.
#[test]
fn test_all_item_quantities_serde() {
    let quantities = vec![
        ItemQuantity::Scarce,
        ItemQuantity::Normal,
        ItemQuantity::Abundant,
    ];

    for qty in quantities {
        let json = serde_json::to_string(&qty).unwrap();
        let deserialized: ItemQuantity = serde_json::from_str(&json).unwrap();
        assert_eq!(qty, deserialized);
    }
}

/// Test that all EventFrequency variants can be serialized/deserialized.
#[test]
fn test_all_event_frequencies_serde() {
    let frequencies = vec![
        EventFrequency::Rare,
        EventFrequency::Normal,
        EventFrequency::Frequent,
    ];

    for freq in frequencies {
        let json = serde_json::to_string(&freq).unwrap();
        let deserialized: EventFrequency = serde_json::from_str(&json).unwrap();
        assert_eq!(freq, deserialized);
    }
}

/// Test that probability values make sense (increase from Rare to Frequent).
#[test]
fn test_event_frequency_ordering() {
    let rare = EventFrequency::Rare.event_probability();
    let normal = EventFrequency::Normal.event_probability();
    let frequent = EventFrequency::Frequent.event_probability();

    assert!(rare < normal, "Rare should be less probable than Normal");
    assert!(
        normal < frequent,
        "Normal should be less probable than Frequent"
    );
    assert!(frequent <= 1.0, "Probability should not exceed 1.0");
    assert!(rare >= 0.0, "Probability should not be negative");
}

/// Test that item counts make sense (increase from Scarce to Abundant).
#[test]
fn test_item_quantity_ordering() {
    let scarce = ItemQuantity::Scarce.base_item_count();
    let normal = ItemQuantity::Normal.base_item_count();
    let abundant = ItemQuantity::Abundant.base_item_count();

    assert!(
        scarce < normal,
        "Scarce should have fewer items than Normal"
    );
    assert!(
        normal < abundant,
        "Normal should have fewer items than Abundant"
    );
    assert!(scarce > 0, "Scarce should have at least 1 item");
}

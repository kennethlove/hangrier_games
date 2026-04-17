#![allow(non_snake_case)]

use dioxus::prelude::*;
use web::components::InfoDetail;

/// Test InfoDetail renders with basic props
#[test]
fn test_info_detail_renders() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            InfoDetail {
                label: "Health".to_string(),
                value: "100".to_string(),
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test InfoDetail with empty value
#[test]
fn test_info_detail_empty_value() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            InfoDetail {
                label: "Status".to_string(),
                value: "".to_string(),
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test InfoDetail with numeric values
#[test]
fn test_info_detail_numeric() {
    let mut dom = VirtualDom::new(|| {
        let health = 75;

        rsx! {
            InfoDetail {
                label: "Health".to_string(),
                value: format!("{}", health),
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test multiple InfoDetail components
#[test]
fn test_multiple_info_details() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            div {
                InfoDetail {
                    label: "Health".to_string(),
                    value: "100".to_string(),
                }
                InfoDetail {
                    label: "Sanity".to_string(),
                    value: "85".to_string(),
                }
                InfoDetail {
                    label: "Movement".to_string(),
                    value: "50".to_string(),
                }
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Multiple components should render without panicking
}

/// Test InfoDetail with long text
#[test]
fn test_info_detail_long_text() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            InfoDetail {
                label: "Description".to_string(),
                value: "This is a very long description that should still render correctly even with lots of text".to_string(),
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

/// Test InfoDetail with special characters
#[test]
fn test_info_detail_special_chars() {
    let mut dom = VirtualDom::new(|| {
        rsx! {
            InfoDetail {
                label: "Special & Characters".to_string(),
                value: "100% <strong>".to_string(),
            }
        }
    });

    let _edits = dom.rebuild_to_vec();
    // Component should build without panicking
}

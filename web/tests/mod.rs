// Component tests for the Dioxus web frontend
//
// These tests verify that components render correctly without panicking.
// Since Dioxus testing is limited for WASM targets, we focus on:
// - Component rendering without errors
// - State management with signals
// - Proper prop handling
//
// Note: These tests run in a non-WASM context, so browser-specific
// features and API calls are not tested here.

mod button_test;

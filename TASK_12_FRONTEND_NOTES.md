# Frontend Terrain UI Changes

## Summary
This document describes the minimal frontend changes made for Task 12 and outlines future enhancements.

## Completed Changes

### 1. Game Areas Display (game_areas.rs)
- Added TODO comment at line 92-95 indicating where terrain display would be added
- Location: `web/src/components/game_areas.rs:92`
- Future enhancement: Display terrain type alongside area name
  Example: `North (Dense Forest)` instead of just `North`

### 2. Game Creation Form (create_game.rs)  
- Added comprehensive TODO comment block at top of file
- Documents exactly how to add ItemQuantity and EventFrequency dropdowns
- Location: `web/src/components/create_game.rs:11-19`
- Includes step-by-step implementation guide

## Future Frontend Enhancements

### Game Creation Customization
To fully implement game customization in the frontend:

1. **Import Types**
   ```rust
   use shared::{ItemQuantity, EventFrequency};
   ```

2. **Add State Signals**
   ```rust
   let mut item_quantity: Signal<ItemQuantity> = use_signal(|| ItemQuantity::Normal);
   let mut event_frequency: Signal<EventFrequency> = use_signal(|| EventFrequency::Normal);
   ```

3. **Create Dropdown Components**
   ```rust
   select {
       class: "border rounded px-2 py-1",
       onchange: move |e| {
           let value = match e.value().as_str() {
               "Scarce" => ItemQuantity::Scarce,
               "Abundant" => ItemQuantity::Abundant,
               _ => ItemQuantity::Normal,
           };
           item_quantity.set(value);
       },
       option { value: "Scarce", "Scarce Items" }
       option { value: "Normal", selected: true, "Normal Items" }
       option { value: "Abundant", "Abundant Items" }
   }
   ```

4. **Modify API Call**
   Update `create_game()` function to send CreateGame DTO:
   ```rust
   let json_body = serde_json::json!({
       "name": name,
       "item_quantity": item_quantity.read().clone(),
       "event_frequency": event_frequency.read().clone(),
       "starting_health_range": None, // or Some((80, 100))
   });
   ```

### Terrain Display Enhancement
To show terrain types in area listings:

1. **Add Terrain Display Function**
   ```rust
   fn format_area_name(area: &AreaDetails) -> String {
       format!("{} ({})", area.name, area.terrain.base_terrain)
   }
   ```

2. **Update Template**
   ```rust
   h4 { "{format_area_name(&area)}" }
   ```

3. **Add Terrain Icons/Colors** (Optional)
   - Desert:  / Sandy yellow background
   - Forest:  / Deep green background  
   - Tundra:  / Icy blue background
   - Mountains:  / Rocky gray background
   - etc.

## Testing Considerations

### Manual Testing
1. Create game with different customization options
2. Verify area displays show terrain types correctly
3. Test that terrain names are properly formatted
4. Ensure dropdown selections persist through form validation

### Visual Regression Testing
- Capture screenshots of game creation form before/after
- Verify area list styling remains consistent
- Test all three themes (theme1, theme2, theme3)

## Notes
- Frontend changes are minimal placeholders due to time constraints
- Full implementation requires coordination with API changes (Task 9)
- Terrain data must be properly populated in backend for display to work
- Consider adding terrain legend/glossary for new players

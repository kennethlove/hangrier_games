# shared/

## Responsibility

Provides shared data types and contracts between the API server (`api/`) and core simulation (`game/`). Ensures consistent serialization/deserialization across the HTTP API and game engine boundary.

## Design

**Type Categories:**

1. **Game Management:** `CreateGame`, `EditGame`, `DeleteGame`, `GameArea`
2. **Game State:** `DisplayGame`, `ListDisplayGame`, `GameStatus`, `CreatedBy`
3. **Tribute Operations:** `EditTribute`, `DeleteTribute`, `TributeKey`
4. **Authentication:** `RegistrationUser`, `AuthenticatedUser`

**Design Patterns:**

- **Tuple Structs for Simple Types:** `DeleteTribute`, `DeleteGame`, `EditTribute` use positional fields (legacy pattern, marked for refactoring to named fields)
- **Enum with Display/FromStr:** `GameStatus` supports both serialization and string parsing with flexible input (`"NotStarted"`, `"not started"`, etc.)
- **Serde Defaults:** Many fields use `#[serde(default)]` to handle missing JSON fields gracefully
- **Derive-heavy:** All types implement `Debug`, `Serialize`, `Deserialize`; most also `Clone`, `PartialEq`

**GameStatus State Machine:**
```
NotStarted → InProgress → Finished
```

## Flow

**Client (HTMX browser / API client) → API:**
- HTML form submissions or JSON requests carry `EditGame`, `EditTribute`, `RegistrationUser`
- API endpoints deserialize into these shared types

**API → Client:**
- Database queries map to `DisplayGame`, `ListDisplayGame`
- Responses serialize to JSON for REST clients, or rendered server-side via Maud templates

**Game Simulation:**
- `game/` crate imports `GameStatus` to track simulation lifecycle
- Status flows: core logic → database → API templates for HTML display

## Integration

**Used By:**
- `api/src/games.rs`: `DisplayGame`, `EditGame`, `GameStatus`, `GameArea`, `ListDisplayGame`
- `api/src/tributes.rs`: `EditTribute`
- `api/src/templates/*.rs`: Maud template components render shared types to HTML
- `game/src/games.rs`: `GameStatus` for simulation state

**Key Files:**
- `lib.rs`: All shared types (single-file crate)

**Notable TODO:**
- `EditTribute` marked for refactoring from tuple to named struct for clarity

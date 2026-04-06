# shared/

## Responsibility

Provides shared data types and contracts between frontend (`web/`), backend (`api/`), and core simulation (`game/`). Ensures consistent serialization/deserialization across the WASM boundary and HTTP API.

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

**Frontend → API:**
- Web components serialize `EditGame`, `EditTribute`, `RegistrationUser` to JSON
- API endpoints deserialize into these shared types

**API → Frontend:**
- Database queries map to `DisplayGame`, `ListDisplayGame`
- Responses serialize to JSON for Dioxus components

**Game Simulation:**
- `game/` crate imports `GameStatus` to track simulation lifecycle
- Status flows: core logic → database → frontend display

## Integration

**Used By:**
- `api/src/games.rs`: `DisplayGame`, `EditGame`, `GameStatus`, `GameArea`, `ListDisplayGame`
- `api/src/tributes.rs`: `EditTribute`
- `web/src/components/*.rs`: Nearly all UI components (15+ files)
- `web/src/cache.rs`: `AuthenticatedUser`, `DisplayGame`, `TributeKey` for local state
- `game/src/games.rs`: `GameStatus` for simulation state

**Key Files:**
- `lib.rs`: All shared types (single-file crate)

**Notable TODO:**
- `EditTribute` marked for refactoring from tuple to named struct for clarity

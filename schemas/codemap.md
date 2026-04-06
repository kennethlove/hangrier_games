# schemas/

## Responsibility

Defines the SurrealDB database schema for the Hangrier Games application. This directory contains the complete data model for game sessions, tributes, items, areas, user authentication, message logs, and summaries. The schemas establish:

- Table structures (SCHEMAFULL vs SCHEMALESS)
- Graph relationships between entities
- Permission-based access control using `$auth` context
- Custom query functions for complex data aggregation
- Game rule enforcement through constraints and validation logic

## Design

### Schema Organization

**8 Schema Files:**
- `game.surql` - Game sessions and core game logic functions
- `tribute.surql` - Player characters (contestants)
- `area.surql` - Arena locations
- `item.surql` - Items (weapons, consumables, etc.)
- `logs.surql` - Immutable event/message log
- `summary.surql` - LLM-generated daily summaries
- `users.surql` - Authentication and user accounts
- `script_migration.surql` - Migration tracking

### Key Patterns

**1. Graph-First Architecture**
SurrealDB's graph database capabilities are extensively used:
```
user ─creates─> game ─areas─> area ─items─> item
                  ↑                           ↑
                  │                           │
                  └─ playing_in ─ tribute ─owns
```

**Relation Tables (Graph Edges):**
- `owns` (tribute→item) - Item possession
- `playing_in` (tribute→game, ENFORCED) - Tribute enrollment
- `areas` (game→area, ENFORCED) - Arena composition
- `items` (area→item) - Item locations
- `summaries` (game→summary) - Summary linkage

**2. Permission Model**
Fine-grained access control at table and field level:
- **Public Read, Owner Write:** `game`, `tribute`
- **Authenticated Only:** `area`, `item`, `summary`
- **Owner Only:** `user` (record-level isolation)
- **Immutable Audit:** `message`, `script_migration` (no UPDATE/DELETE)

**3. Schema Flexibility**
- **SCHEMAFULL:** `user`, `game`, `message`, `script_migration` (strict validation)
- **SCHEMALESS:** `tribute`, `area`, `item`, `summary` (flexible game mechanics)

**4. Custom Query Functions**
Complex queries encapsulated in reusable functions (prefix `fn::`):
- `fn::get_full_game()` - Complete game state with tributes, areas, items
- `fn::get_display_game()` - UI-optimized game data with winner/readiness
- `fn::get_full_tribute()` - Tribute with items and log
- `fn::get_messages_by_*()` - Various message filtering strategies

**5. Game Rule Enforcement**

**Readiness Validation:**
Games are ready when: 24 tributes AND 12 unique districts (2 per district)
```sql
count(<-playing_in<-tribute.id) == 24
AND count(array::distinct(<-playing_in<-tribute.district)) == 12
```

**Winner Determination:**
```sql
IF count($living_tributes) == 1 THEN $living_tributes[0].name
```

**Editability Control:**
Tributes editable only when `game.status == "NotStarted"`

**Audit Integrity:**
Messages/migrations cannot be modified after creation

### Indexes

**Unique Identifiers:**
- `game.identifier`, `tribute.identifier`, `user.username`, `item.identifier`

**Foreign Keys:**
- `tribute.created_by`, relation pairs (game+area, tribute+item, etc.)

**Time-Based:**
- `message.timestamp`, `message.game_day`

**Filtering:**
- `area.name`, `area.area`, `message.source.type`, `message.game_id`

## Flow

### Authentication Flow
1. User signup/signin via JWT (`users.surql` ACCESS configuration)
2. Argon2 password hashing on signup
3. JWT token issued (1 hour expiry, HS512)
4. `$auth` context populated for permission checks

### Game Lifecycle
1. **Setup:** User creates `game` → User creates 24 `tribute` records → Tributes linked via `playing_in` relation
2. **Validation:** `ready` flag calculated (24 tributes, 12 districts)
3. **Execution:** Game core (`game/` crate) runs simulation → API creates `message` records for events
4. **Announcements:** LLM (`announcers/` crate) queries messages → Generates `summary` records
5. **Completion:** Game status updated to "Finished" → Winner determined (last tribute with health > 0)

### Data Access Patterns
**Write Path:**
```
API → SurrealDB tables → Relation updates → Index updates
```

**Read Path (Simple):**
```
API → SELECT with permissions → Direct table read
```

**Read Path (Complex):**
```
API → Custom function (fn::get_*) → Graph traversal → Aggregation → Return
```

**Graph Traversal Examples:**
- `->owns->item[*]` - Get all items owned by tribute
- `<-playing_in<-tribute[*]` - Get all tributes in game
- `->areas->area->items->item` - Get all items in game areas

## Integration

### API Layer (`api/` crate)
- Calls custom functions for queries: `fn::get_full_game()`, `fn::get_display_game()`, etc.
- Manages `$auth` context from JWT tokens
- Creates `message` records for game events
- Persists game state updates

### Game Core (`game/` crate)
- Pure Rust simulation logic (no DB interaction)
- API translates simulation results to database updates
- State synchronized after each simulation step

### Frontend (`web/` crate)
- Authenticates via JWT (signup/signin)
- Queries games via API endpoints
- Real-time updates by polling game state
- Permission checks handled transparently by schema

### Announcers (`announcers/` crate)
- Queries `message` table via `fn::get_messages_by_*()` functions
- Generates narrative summaries using Ollama LLM
- Stores results in `summary` table via `summaries` relation

### Migration System
- `surrealdb-migrations` crate manages schema evolution
- `DEFINE ... OVERWRITE` allows idempotent re-application
- `script_migration` table prevents duplicate execution
- Initial state in `migrations/definitions/_initial.json`

### Security Integration
- Permission checks enforced at database level (not API)
- `created_by` fields auto-populated from `$auth`
- Row-level security: users can only modify their own data
- Immutable logs ensure audit trail integrity

---

## Notable Implementation Details

**JWT Secret:** Hardcoded in `users.surql` - should be environment variable in production

**ENFORCED Relations:** `playing_in` and `areas` use `ENFORCED` constraint to ensure referential integrity

**Nested Queries:** Custom functions use subqueries and `$parent` context for correlated queries

**Array Functions:** `array::distinct()` used for district uniqueness validation

**String Functions:** `string::starts_with()`, `string::contains()` for message filtering

**Graph Syntax:** `->relation->table` for forward traversal, `<-relation<-table` for backward

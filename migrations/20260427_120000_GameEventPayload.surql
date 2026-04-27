-- Persist the structured `GameEvent` payload alongside the existing
-- stringly-typed `content` field on `message` rows (mqi.3). The payload is
-- the externally-tagged JSON form of `game::events::GameEvent`, serialized
-- to a string. Stored as `string` (not `object`) because the SurrealDB
-- SDK serializer collapses externally-tagged Rust enums to `{}` when bound
-- into an `object` column; round-tripping through a string preserves the
-- tagged shape verbatim. Optional so legacy rows and plain `log` calls
-- remain valid. `event_id` is a stable per-row UUID for the structured
-- payload, distinct from the surrounding message `identifier`, so future
-- hashing / dedup of structured events does not have to piggyback on the
-- message row identity.
DEFINE FIELD OVERWRITE event ON message TYPE option<string>;
DEFINE FIELD OVERWRITE event_id ON message TYPE option<string>;
DEFINE INDEX OVERWRITE message_event_id ON message FIELDS event_id;

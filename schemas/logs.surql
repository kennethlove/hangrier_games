DEFINE TABLE OVERWRITE message SCHEMAFULL
    PERMISSIONS
        FOR select FULL
        FOR update, delete NONE
        FOR create WHERE $auth;

DEFINE FIELD OVERWRITE identifier ON message;
DEFINE FIELD OVERWRITE source ON message TYPE object;
DEFINE FIELD OVERWRITE source.type ON message TYPE string;
DEFINE FIELD OVERWRITE source.value ON message TYPE string;
DEFINE FIELD OVERWRITE game_day ON message TYPE number;
DEFINE FIELD OVERWRITE subject ON message TYPE string;
DEFINE FIELD OVERWRITE timestamp ON message TYPE number;
DEFINE FIELD OVERWRITE content ON message TYPE string;
-- Optional typed category for engine-emitted events. See
-- `game::messages::MessageKind`. Legacy rows leave this unset.
DEFINE FIELD OVERWRITE kind ON message TYPE option<string>;
-- Optional structured payload for engine-emitted events. Holds the
-- externally-tagged JSON form of `game::events::GameEvent` serialized to
-- a string. Stored as `string` (not `object`) because SurrealDB's SDK
-- serializer collapses externally-tagged Rust enums and arbitrary
-- `serde_json::Value::Object` payloads to `{}` when binding into an
-- `object` column. Round-tripping through a string preserves the tagged
-- shape verbatim. Legacy rows (and string-only `log` calls) leave this
-- unset.
DEFINE FIELD OVERWRITE event ON message TYPE option<string>;
-- Stable per-row identifier for the structured event payload, distinct
-- from the message `identifier`. Stored as a string (matches the rest of
-- the repo's UUID-on-the-wire convention; native `uuid` columns trip on
-- the wire-format mismatch between the SDK's `uuid::Uuid` and Surreal's
-- own UUID type). Enables future hashing / dedup of structured events
-- without coupling to the surrounding message row.
DEFINE FIELD OVERWRITE event_id ON message TYPE option<string>;

-- Define indexes for querying
DEFINE INDEX OVERWRITE message_timestamp ON message FIELDS timestamp;
DEFINE INDEX OVERWRITE message_game_day ON message FIELDS game_day;
DEFINE INDEX OVERWRITE message_source_type ON message FIELDS source.type;
DEFINE INDEX OVERWRITE message_game_id ON message FIELDS source.value;
DEFINE INDEX OVERWRITE message_kind ON message FIELDS kind;
DEFINE INDEX OVERWRITE message_event_id ON message FIELDS event_id;

-- Helper functions for common queries
DEFINE FUNCTION OVERWRITE fn::get_messages_by_day($day: number) {
    RETURN SELECT * FROM message
    WHERE game_day = $day
    ORDER BY timestamp;
};

DEFINE FUNCTION OVERWRITE fn::get_messages_by_source_type($source_type: string) {
    RETURN SELECT * FROM message
    WHERE source.type = $source_type
    ORDER BY timestamp;
};

DEFINE FUNCTION OVERWRITE fn::get_game_messages_by_game_id($game_id: string) {
    RETURN SELECT * FROM message
    WHERE source.type = 'Game'
    AND source.value = $game_id
    ORDER BY timestamp;
};

DEFINE FUNCTION OVERWRITE fn::get_messages_by_game_id($game_id: string) {
    RETURN SELECT * FROM message
    WHERE string::starts_with(subject, $game_id)
    ORDER BY timestamp;
};

DEFINE FUNCTION OVERWRITE fn::get_messages_by_tribute_id($tribute_id: string) {
    RETURN SELECT * FROM message
    WHERE string::contains(subject, $tribute_id)
    ORDER BY timestamp;
};

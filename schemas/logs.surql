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
-- Period phase ("day" or "night") for this message; see
-- `shared::messages::Phase`. Required for causal ordering.
DEFINE FIELD OVERWRITE phase ON message TYPE string;
-- Per-period monotonic action counter. Resets each phase boundary.
DEFINE FIELD OVERWRITE tick ON message TYPE number;
-- Within-period emit ordinal. Resets each phase boundary alongside `tick`.
DEFINE FIELD OVERWRITE emit_index ON message TYPE number;
-- Structured `MessagePayload` stored as a JSON-encoded `string`. Stored as
-- `string` (not `object`) because SurrealDB's SDK serializer collapses
-- internally-tagged Rust enums and arbitrary `serde_json::Value::Object`
-- payloads to `{}` when binding into an `object` column. Round-tripping
-- through a string preserves the tagged shape verbatim.
DEFINE FIELD OVERWRITE payload ON message TYPE string;

-- Define indexes for querying
DEFINE INDEX OVERWRITE message_timestamp ON message FIELDS timestamp;
DEFINE INDEX OVERWRITE message_game_day ON message FIELDS game_day;
DEFINE INDEX OVERWRITE message_source_type ON message FIELDS source.type;
DEFINE INDEX OVERWRITE message_game_id ON message FIELDS source.value;
DEFINE INDEX OVERWRITE message_phase ON message FIELDS phase;

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

-- Game timeline redesign — PR1 backend (lca):
--
-- Replaces the mqi.3-era `kind`/`event`/`event_id` columns with the new
-- typed payload shape: `phase`/`tick`/`emit_index`/`payload`. The
-- `payload` column stores `MessagePayload` as a JSON-encoded string for
-- the same SDK serializer reasons as the prior `event` column.

REMOVE INDEX message_kind ON message;
REMOVE INDEX message_event_id ON message;
REMOVE FIELD kind ON message;
REMOVE FIELD event ON message;
REMOVE FIELD event_id ON message;

DEFINE FIELD OVERWRITE phase ON message TYPE string;
DEFINE FIELD OVERWRITE tick ON message TYPE number;
DEFINE FIELD OVERWRITE emit_index ON message TYPE number;
DEFINE FIELD OVERWRITE payload ON message TYPE string;

DEFINE INDEX OVERWRITE message_phase ON message FIELDS phase;

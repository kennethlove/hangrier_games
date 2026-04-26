-- Add optional `kind` field to message rows so engine-emitted alliance
-- events can be tagged with a typed `MessageKind` (see
-- `game::messages::MessageKind`). Existing rows have no `kind` and remain
-- valid because the field is optional.
DEFINE FIELD OVERWRITE kind ON message TYPE option<string>;
DEFINE INDEX OVERWRITE message_kind ON message FIELDS kind;

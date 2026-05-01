-- Switch tribute.allies from `array<uuid>` to `array<string>`.
--
-- The Surreal Rust SDK's bespoke serializer wires `uuid::Uuid` as raw bytes,
-- which Surreal then renders as base64 ("c6DKfVvsTRS5kjeYRhFKSQ" etc.) and
-- rejects against an `array<uuid>` column constraint. The repo already
-- documents this convention for `message.event_id` (see migration
-- 20260427_180000_GameMessagePayloadV2): UUIDs ride on the wire as strings,
-- and the application-side serde now serializes `Vec<Uuid>` as a sequence of
-- strings to match.
DEFINE FIELD OVERWRITE allies ON tribute TYPE array<string> DEFAULT [];

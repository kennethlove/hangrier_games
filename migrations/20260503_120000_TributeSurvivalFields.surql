-- Adds the persisted survival-state fields used by the shelter +
-- hunger/thirst system (Plan 1: backend). All fields default-safe so
-- existing rows hydrate cleanly — pre-feature tributes load as Sated and
-- exposed.
--
-- Mirrors the serde(default) annotations on `Tribute` in
-- game/src/tributes/mod.rs.
DEFINE FIELD OVERWRITE hunger                  ON tribute TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE thirst                  ON tribute TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE sheltered_until         ON tribute TYPE option<int>;
DEFINE FIELD OVERWRITE starvation_drain_step   ON tribute TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE dehydration_drain_step  ON tribute TYPE int DEFAULT 0;

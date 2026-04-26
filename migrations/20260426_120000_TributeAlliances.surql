-- Tribute alliances v1: traits replace BrainPersonality;
-- allies + traits + turns_since_last_betrayal added on tribute.
-- Existing tribute rows are not migrated. Reset your dev DB.
DELETE owns;
DELETE playing_in;
DELETE tribute;

DEFINE TABLE tribute_histories SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE game_id IN (SELECT identifier FROM game WHERE created_by.id = $auth.id)
        FOR create, update, delete
            NONE; -- Server-side only

DEFINE FIELD OVERWRITE game_id ON tribute_histories TYPE string;
DEFINE FIELD OVERWRITE digests ON tribute_histories TYPE array;
DEFINE INDEX idx_tribute_histories_game ON tribute_histories COLUMNS game_id UNIQUE;

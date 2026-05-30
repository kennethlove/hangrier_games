DEFINE TABLE commentary_segments SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE game_id IN (SELECT identifier FROM game WHERE created_by.id = $auth.id)
        FOR create
            NONE -- Server-side only
        FOR update, delete
            NONE;

DEFINE FIELD OVERWRITE game_id ON commentary_segments TYPE string;
DEFINE FIELD OVERWRITE day ON commentary_segments TYPE int;
DEFINE FIELD OVERWRITE phase ON commentary_segments TYPE string;
DEFINE FIELD OVERWRITE lines ON commentary_segments TYPE array;
DEFINE FIELD OVERWRITE generated_at ON commentary_segments TYPE datetime;
DEFINE FIELD OVERWRITE model_used ON commentary_segments TYPE string;
DEFINE INDEX idx_commentary_game_day_phase ON commentary_segments COLUMNS game_id, day, phase UNIQUE;

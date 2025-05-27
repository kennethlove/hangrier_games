DEFINE TABLE OVERWRITE game SCHEMAFULL
    PERMISSIONS
        FOR select
            WHERE private = false
            OR created_by.id = $auth.id
        FOR create, update, delete
            WHERE created_by.id = $auth.id;

DEFINE FIELD OVERWRITE identifier ON game;
DEFINE FIELD OVERWRITE name ON game;
DEFINE FIELD OVERWRITE status ON game;
DEFINE FIELD OVERWRITE day ON game;
DEFINE FIELD OVERWRITE created_by ON game VALUE $auth READONLY;
DEFINE FIELD OVERWRITE private ON game DEFAULT true;

DEFINE INDEX OVERWRITE game_identifier ON game FIELDS identifier UNIQUE;

DEFINE TABLE OVERWRITE areas TYPE RELATION IN game OUT area ENFORCED
PERMISSIONS FOR
        SELECT, CREATE, UPDATE, DELETE WHERE $auth;

DEFINE INDEX OVERWRITE game_areas ON areas FIELDS game, area;

DEFINE FUNCTION OVERWRITE fn::get_full_game($game_id: string) {
    RETURN SELECT *, (
        SELECT *, ->owns->item[*] AS items
        FROM <-playing_in<-tribute[*]
        ORDER district
    )
    AS tributes, (
        SELECT *, ->items->item[*] AS items
        FROM ->areas->area
    ) AS areas, (
        RETURN count(SELECT id FROM <-playing_in<-tribute)
    ) AS tribute_count,
    count(<-playing_in<-tribute.id) == 24
    AND
    count(array::distinct(<-playing_in<-tribute.district)) == 12
    AS ready
    FROM game
    WHERE identifier = $game_id;
};

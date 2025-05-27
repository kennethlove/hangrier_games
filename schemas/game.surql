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

DEFINE FUNCTION OVERWRITE fn::get_display_game($game_id: string) {
    LET $tributes = (
        SELECT  in.id,
                in.name,
                in.district,
                in.attributes.health
        FROM playing_in
        WHERE out.identifier = $game_id
    );

    LET $living_tributes = (
        SELECT * FROM $tributes.in WHERE attributes.health > 0
    );

    LET $winner = (
        IF count($living_tributes) == 1 THEN
            RETURN $living_tributes[0].name
        ELSE
            RETURN ""
        END
    );

    RETURN SELECT
        identifier,
        name,
        status,
        day,
        private,
        created_by.username,
        created_by.id == $auth.id AS is_mine,
        $tributes.in AS tributes,
        count($tributes.in) as tribute_count,
        count(SELECT * FROM $tributes.in WHERE attributes.health > 0) AS living_count,
        count($tributes.in) == 24 AND
            count(array::distinct(<-playing_in<-tribute.district)) == 12 AS ready,
        $winner AS winner
    FROM game
    WHERE identifier = $game_id
    LIMIT 1
    FETCH tribute
};

DEFINE FUNCTION OVERWRITE fn::get_list_games() {
    RETURN SELECT name, identifier, status, day, private,
    created_by.id == $auth.id AS is_mine,
    created_by.username,
    (
        RETURN count(
            SELECT id FROM <-playing_in<-tribute
        )
    ) AS tribute_count,
    (
        RETURN count(
            SELECT id FROM <-playing_in<-tribute WHERE attributes.health > 0
        )
    ) AS living_count,
    count(<-playing_in<-tribute.id) == 24
    AND
    count(array::distinct(<-playing_in<-tribute.district)) == 12
    AS ready
    FROM game
};

DEFINE FUNCTION OVERWRITE fn::get_detail_game($game_id: string) {
    LET $day = SELECT day FROM game WHERE identifier = $game_id LIMIT 1;

    RETURN SELECT *, (
        SELECT *,
        ->owns->item[*] AS items, (
            SELECT *
            FROM tribute_log
            WHERE tribute_identifier = $parent.identifier
            AND day = $day
            ORDER BY instant
        ) AS log
        FROM <-playing_in<-tribute[*]
        ORDER district
    ) AS tributes, (
        SELECT *, ->items->item[*] AS items
        FROM ->areas->area
    ) AS areas, (
        RETURN count(SELECT id FROM <-playing_in<-tribute)
    ) AS tribute_count,
    count(<-playing_in<-tribute.id) == 24 AND count(array::distinct(<-playing_in<-tribute.district)) == 12 AS ready,
    created_by.id == $auth.id AS is_mine,
    created_by.username
    FROM game
    WHERE identifier = $game_id
};

DEFINE FUNCTION OVERWRITE fn::get_tributes_by_game($game_id: string) {
    RETURN SELECT (
        SELECT *, ->owns->item[*] AS items
        FROM <-playing_in<-tribute
        ORDER district
    ) AS tributes
    FROM game WHERE identifier = $game_id
};

DEFINE FUNCTION OVERWRITE fn::get_tributes_items_by_game($game_id: string) {
    RETURN SELECT <-playing_in<-tribute as tribute,
        <-playing_in<-tribute->owns->item as item,
        <-playing_in<-tribute->owns as owns
    FROM game WHERE identifier = $game_id
};

DEFINE FUNCTION OVERWRITE fn::get_areas_items_by_game($game_id: string) {
    RETURN SELECT ->areas->area AS area,
        ->areas->area->items->item AS item,
        ->areas->area->items AS items,
        -> areas AS areas
    FROM game WHERE identifier = $game_id;
};

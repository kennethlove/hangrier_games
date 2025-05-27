DEFINE TABLE OVERWRITE tribute SCHEMALESS
    PERMISSIONS
        FOR select FULL
        FOR create, update, delete WHERE created_by = $auth;

DEFINE FIELD OVERWRITE identifier ON tribute;
DEFINE FIELD OVERWRITE status ON tribute;
DEFINE FIELD OVERWRITE avatar ON tribute;
DEFINE FIELD OVERWRITE name ON tribute;
DEFINE FIELD OVERWRITE district ON tribute;
DEFINE FIELD OVERWRITE area ON tribute;
DEFINE FIELD OVERWRITE player_name ON tribute;
DEFINE FIELD OVERWRITE statistics ON tribute;
DEFINE FIELD OVERWRITE attributes ON tribute;
DEFINE FIELD OVERWRITE created_by ON tribute VALUE $auth READONLY;

DEFINE INDEX OVERWRITE tribute_identifier ON tribute FIELDS identifier UNIQUE;
DEFINE INDEX OVERWRITE tribute_created_by ON tribute FIELDS created_by;

DEFINE TABLE OVERWRITE owns TYPE RELATION IN tribute OUT item
PERMISSIONS
        FOR SELECT FULL
        FOR CREATE, UPDATE, DELETE WHERE $auth;

DEFINE INDEX OVERWRITE tribute_owns ON owns FIELDS tribute, item;

DEFINE TABLE OVERWRITE playing_in TYPE RELATION IN tribute OUT game ENFORCED
PERMISSIONS
        FOR SELECT FULL
        FOR CREATE, UPDATE, DELETE WHERE $auth;

DEFINE INDEX OVERWRITE tribute_playing_in ON playing_in FIELDS tribute, game;

DEFINE FUNCTION OVERWRITE fn::get_full_tribute($tribute_id: string) {
    RETURN SELECT *, ->owns->item[*] AS items,
    (SELECT * FROM fn::get_messages_by_tribute_id($tribute_id)) AS log,
    (->playing_in->game.status)[0] == "NotStarted" AS editable
    FROM tribute
    WHERE identifier = $tribute_id;
};

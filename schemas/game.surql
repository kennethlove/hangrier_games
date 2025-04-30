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

DEFINE TABLE OVERWRITE areas TYPE RELATION IN game OUT area ENFORCED
PERMISSIONS FOR
        SELECT, CREATE, UPDATE, DELETE WHERE $auth;

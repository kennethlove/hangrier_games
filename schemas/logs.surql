DEFINE TABLE OVERWRITE game_log SCHEMAFULL;

DEFINE FIELD OVERWRITE game_identifier ON game_log;
DEFINE FIELD OVERWRITE day ON game_log;
DEFINE FIELD OVERWRITE message ON game_log;
DEFINE FIELD OVERWRITE instant ON game_log;

--

DEFINE TABLE OVERWRITE tribute_log SCHEMAFULL;

DEFINE FIELD OVERWRITE identifier ON tribute_log;
DEFINE FIELD OVERWRITE tribute_identifier ON tribute_log;
DEFINE FIELD OVERWRITE day ON tribute_log;
DEFINE FIELD OVERWRITE message ON tribute_log;
DEFINE FIELD OVERWRITE instant ON tribute_log;

DEFINE TABLE OVERWRITE tb_logs TYPE RELATION IN tribute OUT tribute_log ENFORCED;

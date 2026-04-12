-- Refresh tokens for JWT token rotation
DEFINE TABLE OVERWRITE refresh_token SCHEMAFULL
    PERMISSIONS
        -- Only allow authenticated users to query their own tokens
        FOR select, create WHERE $auth
        FOR update, delete WHERE user_id = $auth.id;

DEFINE FIELD OVERWRITE token ON refresh_token TYPE string;
DEFINE FIELD OVERWRITE user_id ON refresh_token TYPE record<user>;
DEFINE FIELD OVERWRITE username ON refresh_token TYPE string;
DEFINE FIELD OVERWRITE expires_at ON refresh_token TYPE datetime;
DEFINE FIELD OVERWRITE revoked ON refresh_token TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE created_at ON refresh_token TYPE datetime DEFAULT time::now();

DEFINE INDEX OVERWRITE unique_token ON refresh_token FIELDS token UNIQUE;
DEFINE INDEX OVERWRITE user_tokens ON refresh_token FIELDS user_id;

-- Add avatar field to user table (optional URL string)
DEFINE FIELD OVERWRITE avatar ON user TYPE string
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

-- Add account_status field (default "active")
DEFINE FIELD OVERWRITE account_status ON user TYPE string DEFAULT "active"
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

-- Add ban_reason field (optional string)
DEFINE FIELD OVERWRITE ban_reason ON user TYPE string DEFAULT NONE;

-- Add banned_at field (optional datetime)
DEFINE FIELD OVERWRITE banned_at ON user TYPE datetime DEFAULT NONE;

-- Re-assert user ACCESS to avoid AccessRecordSignupQueryFailed
-- SurrealDB: adding fields to a SCHEMAFULL table without also redefining
-- the ACCESS causes the signup query to fail. Must redefine ACCESS
-- alongside any field additions. See PR #349 for debug history.
DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE user CONTENT {
            username: $display_name,
            email: $email,
            password: crypto::argon2::generate($password)
        }
    )
    SIGNIN (
        SELECT * FROM user
        WHERE email = $email
        AND crypto::argon2::compare(password, $password)
    )
    WITH JWT
        ALGORITHM HS512
        KEY "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"
    DURATION FOR TOKEN 1h;

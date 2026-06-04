-- Add avatar field to user table (optional URL string)
DEFINE FIELD OVERWRITE avatar ON user TYPE option<string>
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

-- Add account_status field (default "active")
DEFINE FIELD OVERWRITE account_status ON user TYPE string DEFAULT "active"
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

-- Add ban_reason field (optional string)
DEFINE FIELD OVERWRITE ban_reason ON user TYPE option<string>;

-- Add banned_at field (optional datetime)
DEFINE FIELD OVERWRITE banned_at ON user TYPE option<datetime>;

-- Re-assert user ACCESS to ensure signup works after adding fields
-- SurrealDB: fields defined as TYPE string (without DEFAULT)
-- reject NONE when the ACCESS SIGNUP doesn't explicitly set them.
-- Redefine ACCESS alongside field additions for consistency.
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

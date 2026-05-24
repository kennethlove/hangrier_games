-- Adds email-based auth support to the user table.
--
-- Adds email + email_verified fields, a unique email index, and
-- updates the SIGNIN/SIGNUP scope to support dual-mode auth
-- (email OR username) during migration.
--
-- The `username` field is repurposed as the display name; login
-- credentials are now email-based.
DEFINE FIELD OVERWRITE email ON user TYPE string
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

DEFINE FIELD OVERWRITE email_verified ON user TYPE bool DEFAULT false
    PERMISSIONS FOR select FULL FOR update WHERE id = $auth.id;

DEFINE INDEX OVERWRITE unique_email ON user FIELDS email UNIQUE;

-- Redefine the access scope for dual-mode auth
DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE user CONTENT {
            username: $display_name ?? $username,
            email: $email,
            password: crypto::argon2::generate($password)
        }
    )
    SIGNIN (
        SELECT * FROM user
        WHERE (email = $email OR username = $email)
        AND crypto::argon2::compare(password, $password)
    )
    WITH JWT
        ALGORITHM HS512
        KEY "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"
    DURATION FOR TOKEN 1h;

DEFINE TABLE OVERWRITE user SCHEMAFULL
    -- Only allow users to query their own record, including their password.
    PERMISSIONS
        FOR select WHERE $auth
        FOR update, delete WHERE id = $auth.id;

DEFINE FIELD OVERWRITE username ON user TYPE string
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id;
DEFINE FIELD OVERWRITE email ON user TYPE string
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id;
DEFINE FIELD OVERWRITE email_verified ON user TYPE bool DEFAULT false
    PERMISSIONS
        FOR select FULL
        FOR update WHERE id = $auth.id;
DEFINE FIELD OVERWRITE password on user TYPE string;

DEFINE INDEX OVERWRITE unique_username ON user FIELDS username UNIQUE;
DEFINE INDEX OVERWRITE unique_email ON user FIELDS email UNIQUE;

DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD
    -- SIGNUP accepts $display_name (new-style) OR $username (old-style) for
    -- backward compatibility during migration. $email is optional for old-style
    -- registrations (existing code that doesn't pass it).
    SIGNUP (
        CREATE user CONTENT {
            username: $display_name ?? $username,
            email: $email,
            password: crypto::argon2::generate($password)
        }
    )
    -- Dual-mode SIGNIN: accepts email OR username in the $email param.
    -- During migration, existing users can still sign in with their username
    -- until they set an email address on their account.
    SIGNIN (
        SELECT * FROM user
        WHERE (email = $email OR username = $email)
        AND crypto::argon2::compare(password, $password)
    )
    WITH JWT
        ALGORITHM HS512
        KEY "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"
    DURATION FOR TOKEN 1h;

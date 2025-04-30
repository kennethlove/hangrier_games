DEFINE TABLE OVERWRITE user SCHEMAFULL
    -- Only allow users to query their own record, including their password.
    PERMISSIONS
        FOR select WHERE $auth
        FOR update, delete WHERE id = $auth.id;

DEFINE FIELD OVERWRITE username ON user TYPE string
    PERMISSIONS
        FOR select FULL
        FOR update, delete WHERE id = $auth.id;
DEFINE FIELD OVERWRITE password on user TYPE string;

DEFINE INDEX OVERWRITE unique_username ON user FIELDS username UNIQUE;

DEFINE ACCESS OVERWRITE user ON DATABASE TYPE RECORD
    SIGNUP (
        CREATE user CONTENT {
            username: $username,
            password: crypto::argon2::generate($password)
        }
    )
    SIGNIN (
        SELECT * FROM user
        WHERE username = $username
        AND crypto::argon2::compare(password, $password)
    )
    WITH JWT
        ALGORITHM HS512
        KEY "6dxLjU0m8ZmAzaLEk_qAeMpeD5ZAjGYlCjlvDi5DcgdJLATIHuCReUu7CbGyCDhRSp3btd7Ezob7RPYe6fUtsA"
    DURATION FOR TOKEN 30d, FOR SESSION 1h;

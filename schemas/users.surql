DEFINE TABLE OVERWRITE user SCHEMAFULL
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
DEFINE FIELD OVERWRITE avatar ON user TYPE string DEFAULT NONE;
DEFINE FIELD OVERWRITE account_status ON user TYPE string DEFAULT "active";
DEFINE FIELD OVERWRITE ban_reason ON user TYPE string DEFAULT NONE;
DEFINE FIELD OVERWRITE banned_at ON user TYPE datetime DEFAULT NONE;

DEFINE INDEX OVERWRITE unique_email ON user FIELDS email UNIQUE;

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

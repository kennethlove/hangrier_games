-- Add avatar field to user table (optional URL string)
DEFINE FIELD OVERWRITE avatar ON user TYPE string DEFAULT NONE;

-- Add account_status field (default "active")
DEFINE FIELD OVERWRITE account_status ON user TYPE string DEFAULT "active";

-- Add ban_reason field (optional string)
DEFINE FIELD OVERWRITE ban_reason ON user TYPE string DEFAULT NONE;

-- Add banned_at field (optional datetime)
DEFINE FIELD OVERWRITE banned_at ON user TYPE datetime DEFAULT NONE;

-- CI trigger

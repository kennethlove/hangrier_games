-- Remove unique_username index. Login is email-based; display names
-- don't need uniqueness and there are no lookup queries by username.
REMOVE INDEX unique_username ON user;

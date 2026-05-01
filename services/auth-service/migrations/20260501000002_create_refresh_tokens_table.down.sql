-- Drop indexes
DROP INDEX IF EXISTS idx_refresh_tokens_user_id ON refresh_tokens;
DROP INDEX IF EXISTS idx_refresh_tokens_expires_at ON refresh_tokens;

-- Drop refresh tokens table
DROP TABLE IF EXISTS refresh_tokens;

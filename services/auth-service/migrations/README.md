# Database Migrations

This directory contains SQLx migrations for the auth-service.

## Prerequisites

Install sqlx-cli with MySQL support:

```bash
cargo install sqlx-cli --no-default-features --features mysql
```

## Setup

1. Create a `.env` file in the auth-service directory (copy from `.env.example`):

```bash
cp .env.example .env
```

2. Update the `DATABASE_URL` in `.env` if needed:

```
DATABASE_URL=mysql://root:password@localhost:3306/auth_db
```

3. Start the MySQL database (using docker-compose):

```bash
cd ../../infrastructure
docker-compose up -d mysql
```

## Running Migrations

### Apply all pending migrations

```bash
cd services/auth-service
sqlx migrate run
```

### Revert the last migration

```bash
sqlx migrate revert
```

### Check migration status

```bash
sqlx migrate info
```

## Creating New Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Example:
sqlx migrate add create_user_roles_table
```

This will create two files:
- `<timestamp>_<migration_name>.up.sql` - Applied when running migrations
- `<timestamp>_<migration_name>.down.sql` - Applied when reverting migrations

## Existing Migrations

1. **20260501000001_create_users_table** - Core users table with authentication fields (id, email, password_hash, is_active, created_at)
2. **20260501000002_create_refresh_tokens_table** - JWT refresh token management with automatic cleanup on user deletion

## SQLx Offline Mode (for Docker builds)

To support Docker builds without database access:

```bash
# Prepare SQLx offline data
cargo sqlx prepare --database-url=$DATABASE_URL

# This creates .sqlx/ directory with compile-time query metadata
```

Add to `.gitignore`:
```
.env
```

Commit `.sqlx/` to source control for offline compilation.

#!/bin/bash
set -e

echo "Waiting for database to be ready..."
# Simple wait for MySQL to be available
sleep 5

# Note: In production, use a proper migration tool or service
# For now, migrations should be run separately using sqlx-cli
# Example: sqlx migrate run --database-url $DATABASE_URL

echo "Starting auth-service..."
exec /app/auth-service

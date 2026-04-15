#!/bin/bash
set -euo pipefail
# Load .env
source .env 2>/dev/null || true
DB_URL="${DATABASE_URL:-postgres://focusflow:focusflow_dev@localhost:5432/focusflow}"
echo "Running migrations against $DB_URL..."
for f in migrations/*.sql; do
    echo "Applying $f..."
    psql "$DB_URL" -f "$f"
done
echo "All migrations applied."

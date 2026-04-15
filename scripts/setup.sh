#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${PROJECT_ROOT}"

echo "==> FocusFlow dev setup"

if ! command -v docker >/dev/null 2>&1; then
  echo "Error: docker is not installed or not on PATH." >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo is not installed or not on PATH." >&2
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "Error: psql is not installed or not on PATH (PostgreSQL client required for migrations)." >&2
  exit 1
fi

echo "==> Rust toolchain"
rustc --version
cargo --version

if [[ ! -f "${PROJECT_ROOT}/.env" ]]; then
  echo "==> Creating .env from .env.example"
  cp "${PROJECT_ROOT}/.env.example" "${PROJECT_ROOT}/.env"
else
  echo "==> .env already exists; skipping copy"
fi

echo "==> Starting Docker services"
docker compose up -d

echo "==> Waiting for PostgreSQL"
until docker compose exec -T postgres pg_isready -U focusflow -d focusflow >/dev/null 2>&1; do
  sleep 1
done

echo "==> Running SQL migrations"
export PGPASSWORD=focusflow_dev
shopt -s nullglob
for sql in "${PROJECT_ROOT}/migrations"/*.sql; do
  echo "    Applying $(basename "${sql}")"
  psql -h localhost -p 5432 -U focusflow -d focusflow -v ON_ERROR_STOP=1 -f "${sql}"
done
shopt -u nullglob
unset PGPASSWORD

echo ""
echo "Setup complete."
echo ""
echo "Next steps:"
echo "  - Review and edit ${PROJECT_ROOT}/.env (API keys, secrets)."
echo "  - Build and run the API: cd backend && cargo run -p focusflow-api (or docker build -f backend/Dockerfile -t focusflow-api .)."
echo "  - Services: Postgres :5432, Redis :6379, Qdrant :6333 (REST) / :6334 (gRPC), NATS :4222 (JetStream), NATS monitoring :8222"
echo "  - Stop stack: docker compose down"

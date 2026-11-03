#!/bin/sh
# Start the bench and Postgres only when their TCP ports are free,
# or already held by these containers.
set -eu
cd "$(dirname "$0")/.."

env_get() {
  key=$1
  def=$2
  if [ -f .env ]; then
    val=$(grep -E "^${key}=" .env | tail -1 | cut -d= -f2- | tr -d '[:space:]')
    if [ -n "$val" ]; then
      printf '%s' "$val"
      return
    fi
  fi
  printf '%s' "$def"
}

require_port() {
  port=$1
  name=$2
  if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
    return 0
  fi
  if docker ps --filter "name=^/${name}$" --format '{{.Ports}}' | grep -q ":${port}->"; then
    return 0
  fi
  echo "TCP $port is already in use. Not starting $name."
  lsof -nP -iTCP:"$port" -sTCP:LISTEN
  exit 1
}

DASH_PORT=$(env_get PRIMNESS_DASHBOARD_PORT 9477)
PG_PORT=$(env_get POSTGRES_PORT 5455)
require_port "$DASH_PORT" primness-bench
require_port "$PG_PORT" primness-db

mkdir -p data/metrics data/hardware
export HOST_UID="$(id -u)"
export HOST_GID="$(id -g)"
docker compose up -d --build
sh scripts/apply-migrations.sh
echo "Primness bench: http://127.0.0.1:${DASH_PORT}/"
echo "Primness Postgres: 127.0.0.1:${PG_PORT} database $(env_get POSTGRES_DB primness)"

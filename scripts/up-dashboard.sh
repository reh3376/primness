#!/bin/sh
# Bind the bench only when its TCP port is free.
set -eu
cd "$(dirname "$0")/.."
PORT=9477
if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "TCP $PORT is already in use. Not starting the bench."
  lsof -nP -iTCP:"$PORT" -sTCP:LISTEN
  exit 1
fi
mkdir -p data/metrics
export HOST_UID="$(id -u)"
export HOST_GID="$(id -g)"
docker compose up -d --build
echo "Primness bench: http://127.0.0.1:${PORT}/"

#!/bin/sh
# Apply db/migrations in order. Safe to run more than once.
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

user=$(env_get POSTGRES_USER primness)
db=$(env_get POSTGRES_DB primness)

i=0
while [ "$i" -lt 30 ]; do
  if docker exec primness-db pg_isready -U "$user" -d "$db" >/dev/null 2>&1; then
    break
  fi
  i=$((i + 1))
  sleep 1
done
docker exec primness-db pg_isready -U "$user" -d "$db" >/dev/null

docker exec primness-db psql -v ON_ERROR_STOP=1 -U "$user" -d "$db" -c \
  "CREATE TABLE IF NOT EXISTS schema_migrations (version TEXT PRIMARY KEY, applied_at TIMESTAMPTZ NOT NULL DEFAULT now());" \
  >/dev/null

for file in db/migrations/*.sql; do
  [ -f "$file" ] || continue
  version=$(basename "$file")
  applied=$(docker exec primness-db psql -tA -U "$user" -d "$db" \
    -c "SELECT 1 FROM schema_migrations WHERE version = '$version'")
  if [ "$applied" = "1" ]; then
    echo "migration $version already applied"
    continue
  fi
  echo "applying $version"
  docker exec -i primness-db psql -v ON_ERROR_STOP=1 -U "$user" -d "$db" < "$file"
  docker exec primness-db psql -v ON_ERROR_STOP=1 -U "$user" -d "$db" \
    -c "INSERT INTO schema_migrations (version) VALUES ('$version')" >/dev/null
done

sql=$(mktemp)
trap 'rm -f "$sql"' EXIT
python3 - <<'PY' > "$sql"
import json, pathlib, secrets
delim = "sch" + secrets.token_hex(8)
for path in sorted(pathlib.Path("schemas").rglob("*.json")):
    doc = json.loads(path.read_text())
    ident = doc["$id"].replace("'", "''")
    payload = json.dumps(doc, separators=(",", ":"))
    marker = f"${delim}$"
    if marker in payload:
        raise SystemExit(f"delimiter collision in {path}")
    print(
        "INSERT INTO schema_documents (schema_version, document) "
        f"VALUES ('{ident}', {marker}{payload}{marker}::jsonb) "
        "ON CONFLICT (schema_version) DO UPDATE SET document = EXCLUDED.document;"
    )
PY
docker exec -i primness-db psql -v ON_ERROR_STOP=1 -U "$user" -d "$db" < "$sql" >/dev/null
echo "schema documents registered"

#!/usr/bin/env bash
# scripts/generate-schema.sh
#
# Regenerate backend-2fa/schema.sql from the concatenated up migrations.
# Run this whenever a new migration is added.
#
# Usage:  bash scripts/generate-schema.sh
#
# The output is written to backend-2fa/schema.sql.  The script preserves
# the header comment block but regenerates all SQL below it.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
MIGRATIONS_DIR="$PROJECT_DIR/migrations"
SCHEMA_FILE="$PROJECT_DIR/schema.sql"

# Temporary files
HEADER_FILE=$(mktemp)
BODY_FILE=$(mktemp)

cleanup() {
    rm -f "$HEADER_FILE" "$BODY_FILE"
}
trap cleanup EXIT

# Extract the existing header (lines until the first blank line after the last comment block).
# We keep everything from the start up to and including the line that says:
# "The migrations/ directory is the single source of truth for the database schema."
awk '
    /single source of truth/ { print; done=1; next }
    done == 1 && /^$/ { print; exit }
    { print }
' "$SCHEMA_FILE" > "$HEADER_FILE"

# Build the body by concatenating up-migration files in version order.
# We sort by the numeric prefix so that 001 comes before 002 etc.
{
    echo ""
    echo "-- ---------------------------------------------------------------------------"
    echo "-- NOTE: This section is auto-generated from the migration files."
    echo "-- Do not edit by hand. Run scripts/generate-schema.sh to regenerate."
    echo "-- ---------------------------------------------------------------------------"
    echo ""

    for f in $(ls "$MIGRATIONS_DIR"/*.sql | sort); do
        basename=$(basename "$f")
        # Skip down-migration scripts
        case "$basename" in
            *.down.sql) continue ;;
            *create_schema_migrations.sql)
                # Include schema_migrations table creation
                cat "$f"
                echo ""
                ;;
            *create_base_tables.sql)
                cat "$f"
                echo ""
                ;;
            *create_ip_access_list.sql)
                cat "$f"
                echo ""
                ;;
            *add_user_two_factor_algorithm.sql)
                # The base table already includes algorithm column, but we include
                # this to document the migration history
                echo "-- $basename (included for migration tracking; column already in CREATE TABLE)"
                echo ""
                ;;
            *encrypt_existing_secrets.sql)
                # Include the SELECT 1 as documentation
                echo "-- $basename"
                cat "$f"
                echo ""
                ;;
        esac
    done

    # two_fa_lockouts is not yet migration-tracked; include it manually
    echo "-- Table: two_fa_lockouts (not yet migration-tracked)"
    echo "CREATE TABLE IF NOT EXISTS two_fa_lockouts ("
    echo "    user_id VARCHAR(255) PRIMARY KEY,"
    echo "    failed_attempts INT NOT NULL DEFAULT 0,"
    echo "    locked BOOLEAN NOT NULL DEFAULT FALSE,"
    echo "    locked_at TIMESTAMP NULL,"
    echo "    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP"
    echo ");"
} > "$BODY_FILE"

# Write the final schema.sql
cat "$HEADER_FILE" "$BODY_FILE" > "$SCHEMA_FILE"
echo "Regenerated $SCHEMA_FILE"

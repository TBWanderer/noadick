#!/bin/bash
set -e

STORAGE_PATH="${STORAGE_PATH:-./storage}"

echo "🔍 Checking for JSON files to migrate in: $STORAGE_PATH"

mkdir -p "$STORAGE_PATH"

json_count=0
migrated_count=0

for json_file in "$STORAGE_PATH"/*.json; do
    [ -e "$json_file" ] || continue
    
    if [[ "$json_file" == *.bak ]]; then
        continue
    fi
    
    json_count=$((json_count + 1))
    echo "📦 Migrating: $json_file"
    
    if migrate_json_to_bin "$json_file"; then
        migrated_count=$((migrated_count + 1))
        echo "✓ Successfully migrated: $json_file"
    else
        echo "✗ Failed to migrate: $json_file"
    fi
done

if [ $json_count -eq 0 ]; then
    echo "✓ No JSON files found to migrate"
elif [ $migrated_count -eq $json_count ]; then
    echo "✓ Successfully migrated all $migrated_count files"
else
    echo "⚠ Migrated $migrated_count out of $json_count files"
fi

echo ""
echo "🚀 Starting application..."
exec "$@"

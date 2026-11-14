#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
    printf 'Usage: %s ISO_3166_1_JSON SOURCE_VERSION\n' "$0" >&2
    exit 2
fi

source_json=$1
source_version=$2
if [[ ! "$source_version" =~ ^[A-Za-z0-9._-]+$ ]]; then
    printf 'Source version may contain only letters, digits, dot, underscore, and hyphen\n' >&2
    exit 2
fi
if [[ ! -f "$source_json" ]]; then
    printf 'Country source JSON was not found: %s\n' "$source_json" >&2
    exit 2
fi

jq -e '
    .["3166-1"] as $territories
    | ($territories | type == "array" and length == 249)
      and all($territories[]; (.alpha_2 | type == "string" and test("^[A-Z]{2}$")))
      and all($territories[]; (.alpha_3 | type == "string" and test("^[A-Z]{3}$")))
      and all($territories[]; (.name | type == "string" and length > 0))
      and (($territories | map(.alpha_2) | unique | length) == ($territories | length))
      and (($territories | map(.alpha_3) | unique | length) == ($territories | length))
' "$source_json" >/dev/null

snapshot_file="$(dirname "$0")/../data/reference/countries-iso-codes-${source_version}.json"
mkdir -p "$(dirname "$snapshot_file")"
jq -S --arg source_version "$source_version" '{
    source: "iso-codes",
    source_version: $source_version,
    records: [.["3166-1"][] | {
        alpha_2: .alpha_2,
        alpha_3: .alpha_3,
        display_name: .name,
        official_name: (.official_name // null)
    }] | sort_by(.alpha_2)
}' "$source_json" > "$snapshot_file"
printf 'Wrote %s (249 entries)\n' "$snapshot_file"

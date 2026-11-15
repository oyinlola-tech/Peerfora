#!/usr/bin/env sh
set -eu
target_app=${1:-.}
output_name=${2:-peerfora-web}
mkdir -p "$target_app/target/site/pkg"
tailwindcss -i style/tailwind.css -o "$target_app/target/site/pkg/$output_name.css" --watch

#!/usr/bin/env sh
set -eu
mkdir -p target/site/pkg
tailwindcss -i style/tailwind.css -o target/site/pkg/peerfora-web.css --minify

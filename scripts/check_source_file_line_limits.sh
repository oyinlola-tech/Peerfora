#!/usr/bin/env bash
set -euo pipefail

root_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
line_limit=400
failed=0

while IFS= read -r -d '' source_file; do
    relative_path="${source_file#"$root_directory"/}"
    line_count="$(awk 'END { print NR }' "$source_file")"

    if (( line_count > line_limit )); then
        printf '%s has %s lines; the limit is %s.\n' \
            "$relative_path" "$line_count" "$line_limit" >&2
        failed=1
    fi
done < <(find "$root_directory" \
    -type d \( -name node_modules -o -name target -o -name .git -o -name vendor \) -prune -o \
    -type d \( -name .gradle -o -name .terraform -o -name .dart_tool -o -name Pods \
        -o -name build -o -name dist -o -name .next -o -name .turbo \) -prune -o \
    -type f ! -name pnpm-lock.yaml ! -name package-lock.json ! -name yarn.lock \
    ! -name Cargo.lock ! -name gradle.lockfile \( -name '*.rs' -o -name '*.sql' \
        -o -name '*.ts' -o -name '*.tsx' \
        -o -name '*.js' -o -name '*.jsx' -o -name '*.py' -o -name '*.sh' \
        -o -name '*.kt' -o -name '*.kts' -o -name '*.gradle' -o -name '*.swift' \
        -o -name '*.dart' -o -name '*.cs' -o -name '*.qml' -o -name '*.css' \
        -o -name '*.scss' -o -name '*.sass' -o -name '*.html' -o -name '*.xml' \
        -o -name '*.bat' -o -name '*.ps1' -o -name '*.yml' -o -name '*.yaml' \
        -o -name '*.toml' -o -name '*.tf' -o -name '*.hcl' -o -name '*.properties' \
        -o -name '*.conf' -o -name '*.ini' -o -name '*.go' -o -name '*.c' \
        -o -name '*.h' -o -name '*.cpp' -o -name '*.hpp' -o -name '*.vue' \
        -o -name '*.svelte' -o -name Dockerfile -o -name 'Dockerfile.*' \
        -o -name Makefile -o -name makefile -o -name Justfile -o -name justfile \
        -o -name gradlew \) -print0)

if (( failed != 0 )); then
    exit 1
fi

printf 'Repository source, configuration, tests, scripts, and migrations are within the %s-line limit.\n' "$line_limit"

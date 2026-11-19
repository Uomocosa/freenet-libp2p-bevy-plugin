#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
cd "$repo_root"

mapfile -t members < <(
    awk '
        /^members = \[/ { in_members = 1; next }
        in_members && /^\]/ { in_members = 0; next }
        in_members {
            gsub(/[",]/, "", $0)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
            if (length($0) > 0) print
        }
    ' Cargo.toml
)

package_name_of() {
    awk -F'"' '
        /^\[package\]/ { in_package = 1; next }
        in_package && /^\[/ { exit }
        in_package && /^name = "/ { print $2; exit }
    ' "$1/Cargo.toml"
}

filters=()

add_package_filter() {
    filters+=("package($1)")
}

add_root_filter() {
    filters+=("(package($1) & test(/^tests::/))")
}

add_module_filter() {
    filters+=("(package($1) & test(/^$2::tests::/))")
}

add_plugin_filters_in() {
    local member="$1" parent_dir="$2" package
    package="$(package_name_of "$member")"
    local plugin
    for plugin in "$member/$parent_dir"/*_plugin.rs; do
        [ -e "$plugin" ] || continue
        local stem="${plugin##*/}"
        stem="${stem%.rs}"
        local modpath="${parent_dir#src/}"
        if [ "$modpath" = "$parent_dir" ] || [ -z "$modpath" ]; then
            modpath="$stem"
        else
            modpath="${modpath//\//::}::$stem"
        fi
        add_module_filter "$package" "$modpath"
    done
}

add_file_filter() {
    local file="$1"
    file="${file#./}"
    local member="" best=0
    for candidate in "${members[@]}"; do
        case "$file" in
            "$candidate"/*)
                if [ "${#candidate}" -gt "$best" ]; then
                    member="$candidate"
                    best="${#candidate}"
                fi
                ;;
        esac
    done
    [ -n "$member" ] || return 0

    local rel="${file#"$member"/}"
    local package
    package="$(package_name_of "$member")"

    case "$rel" in
        src/lib.rs|src/main.rs)
            add_root_filter "$package"
            return 0
            ;;
        src/__basic__/*|src/*/__basic__/*)
            local domain="${rel%%__basic__/*}"
            domain="${domain%/}"
            if [ "$domain" = "src" ] || [ -z "$domain" ]; then
                add_root_filter "$package"
            else
                local module="${domain#src/}"; module="${module//__app__/app}"; module="${module//__fakes__/fakes}"; module="${module//__fixtures__/fixtures}"; module="${module//__queries__/queries}"; add_module_filter "$package" "${module//\//::}"
            fi
            return 0
            ;;
        __methods__/*)
            local method_path="${rel#__methods__/}"
            method_path="${method_path%.rs}"
            add_module_filter "$package" "methods::${method_path//\//::}"
            return 0
            ;;
        src/*.rs)
            ;;
        *)
            return 0
            ;;
    esac

    local modpath="${rel#src/}"
    modpath="${modpath%.rs}"
    modpath="${modpath//__bevy_systems__/bevy_systems}"
    modpath="${modpath//__traits__/traits}"
    modpath="${modpath//__methods__/methods}"
    modpath="${modpath//__basic__/basic}"
    modpath="${modpath//__app__/app}"
    modpath="${modpath//__fakes__/fakes}"
    modpath="${modpath//__fixtures__/fixtures}"
    modpath="${modpath//__queries__/queries}"
    case "$modpath" in
        */mod) modpath="${modpath%/mod}" ;;
    esac
    if [ -z "$modpath" ] || [ "$modpath" = "mod" ]; then
        add_package_filter "$package"
    else
        add_module_filter "$package" "${modpath//\//::}"
    fi

    case "$rel" in
        */__bevy_systems__/*|*/bevy_systems/*)
            local systems_dir="${rel%/*}"
            local parent="${systems_dir%/*}"
            add_plugin_filters_in "$member" "$parent"
            ;;
    esac
}

if [ "$#" -gt 0 ]; then
    for file in "$@"; do
        add_file_filter "$file"
    done
else
    mapfile -t changed < <(
        {
            git diff --name-only HEAD -- '*.rs'
            git ls-files --others --exclude-standard -- '*.rs'
        } | sort -u
    )
    for file in "${changed[@]:-}"; do
        [ -n "$file" ] || continue
        add_file_filter "$file"
    done
fi

unique_filters=()
if [ "${#filters[@]}" -gt 0 ]; then
    mapfile -t unique_filters < <(printf '%s\n' "${filters[@]}" | sort -u)
fi

if [ "${#unique_filters[@]}" -eq 0 ]; then
    printf 'none()\n'
else
    printf '%s' "${unique_filters[0]}"
    for ((i = 1; i < ${#unique_filters[@]}; i++)); do
        printf ' | %s' "${unique_filters[$i]}"
    done
    printf '\n'
fi

#!/usr/bin/env sh
# smoke.sh - runtime smoke harness for petit_trad
#
# Runs model-free runtime checks plus an optional model-backed translation smoke.
# Prints PASS/FAIL/SKIP per check and a final summary.

set -u

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0
TOTAL_COUNT=0
TMP_FILES=""
LAST_STATUS=0
LAST_OUTPUT_FILE=""

SELF_DIR=$(
    CDPATH= cd -- "$(dirname -- "$0")" && pwd
)
REPO_ROOT=$(
    CDPATH= cd -- "$SELF_DIR/.." && pwd
)
cd "$REPO_ROOT"

make_tmp() {
    tmp_file=$(mktemp "${TMPDIR:-/tmp}/petit-smoke.XXXXXX")
    TMP_FILES="${TMP_FILES} ${tmp_file}"
    printf '%s\n' "$tmp_file"
}

cleanup() {
    for file in $TMP_FILES; do
        [ -n "$file" ] && [ -f "$file" ] && rm -f "$file"
    done
}

trap cleanup EXIT INT TERM

print_output_excerpt() {
    file=$1
    max_lines=${2:-12}
    line_no=0
    while IFS= read -r line; do
        line_no=$((line_no + 1))
        if [ "$line_no" -gt "$max_lines" ]; then
            printf '      ...\n'
            return
        fi
        printf '      %s\n' "$line"
    done <"$file"
}

record_status() {
    status=$1
    name=$2
    detail=${3:-}

    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    case "$status" in
        PASS) PASS_COUNT=$((PASS_COUNT + 1)) ;;
        FAIL) FAIL_COUNT=$((FAIL_COUNT + 1)) ;;
        SKIP) SKIP_COUNT=$((SKIP_COUNT + 1)) ;;
        *)
            printf 'Internal error: unknown status %s\n' "$status" >&2
            exit 2
            ;;
    esac

    if [ -n "$detail" ]; then
        printf '%s  %s (%s)\n' "$status" "$name" "$detail"
    else
        printf '%s  %s\n' "$status" "$name"
    fi
}

record_fail_with_output() {
    name=$1
    reason=$2
    cmd=$3
    file=$4

    record_status "FAIL" "$name" "$reason"
    printf '      cmd: %s\n' "$cmd"
    if [ -s "$file" ]; then
        print_output_excerpt "$file" 12
    else
        printf '      (no output)\n'
    fi
}

run_shell_capture() {
    cmd=$1
    LAST_OUTPUT_FILE=$(make_tmp)
    if sh -c "$cmd" >"$LAST_OUTPUT_FILE" 2>&1; then
        LAST_STATUS=0
    else
        LAST_STATUS=$?
    fi
}

run_expect_success() {
    name=$1
    cmd=$2
    expected=${3:-}

    run_shell_capture "$cmd"

    if [ "$LAST_STATUS" -ne 0 ]; then
        record_fail_with_output "$name" "exit $LAST_STATUS" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    if [ -n "$expected" ] && ! grep -Fq -- "$expected" "$LAST_OUTPUT_FILE"; then
        record_fail_with_output "$name" "missing text: $expected" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    record_status "PASS" "$name"
}

run_expect_failure() {
    name=$1
    cmd=$2
    expected=$3

    run_shell_capture "$cmd"

    if [ "$LAST_STATUS" -eq 0 ]; then
        record_fail_with_output "$name" "expected non-zero exit" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    if ! grep -Fq -- "$expected" "$LAST_OUTPUT_FILE"; then
        record_fail_with_output "$name" "missing text: $expected" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    record_status "PASS" "$name" "expected failure"
}

default_model_path_from_config() {
    awk '
        BEGIN { in_model = 0 }
        /^\[model\]/ { in_model = 1; next }
        /^\[/ { in_model = 0 }
        in_model && /^[[:space:]]*path[[:space:]]*=/ {
            line = $0
            sub(/^[^"]*"/, "", line)
            sub(/".*$/, "", line)
            print line
            exit
        }
    ' config/default.toml
}

resolve_model_path() {
    candidate=${SMOKE_MODEL_PATH:-}
    if [ -z "$candidate" ] && [ -f config/default.toml ]; then
        candidate=$(default_model_path_from_config)
    fi

    if [ -z "$candidate" ]; then
        printf '\n'
        return
    fi

    case "$candidate" in
        /*) printf '%s\n' "$candidate" ;;
        *) printf '%s/%s\n' "$REPO_ROOT" "$candidate" ;;
    esac
}

run_optional_model_smoke() {
    model_path=$(resolve_model_path)
    if [ -z "$model_path" ]; then
        record_status "SKIP" "stdin translation smoke" "no model path found"
        return
    fi

    if [ ! -f "$model_path" ]; then
        record_status "SKIP" "stdin translation smoke" "model not found: $model_path"
        return
    fi

    export SMOKE_MODEL_FILE="$model_path"
    cmd='printf "Hello\n" | cargo run --quiet -p petit-tui -- --stdin --model "$SMOKE_MODEL_FILE" --src en --tgt fr --gpu-layers 0'
    run_shell_capture "$cmd"
    unset SMOKE_MODEL_FILE

    if [ "$LAST_STATUS" -ne 0 ]; then
        record_fail_with_output "stdin translation smoke" "exit $LAST_STATUS" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    if [ ! -s "$LAST_OUTPUT_FILE" ]; then
        record_fail_with_output "stdin translation smoke" "empty output" "$cmd" "$LAST_OUTPUT_FILE"
        return
    fi

    record_status "PASS" "stdin translation smoke" "model-backed"
}

printf 'Runtime Smoke Harness (petit_trad)\n'
printf 'Working directory: %s\n' "$REPO_ROOT"
printf '\n'

run_expect_success \
    "help output" \
    'cargo run --quiet -p petit-tui -- --help' \
    'Usage:'

run_expect_success \
    "version output" \
    'cargo run --quiet -p petit-tui -- --version' \
    'petit '

run_expect_failure \
    "unknown flag validation" \
    'cargo run --quiet -p petit-tui -- --definitely-invalid-flag' \
    'Unknown argument: --definitely-invalid-flag'

run_expect_failure \
    "conflicting flag validation" \
    'cargo run --quiet -p petit-tui -- --no-config --config config/default.toml' \
    '--no-config cannot be used with --config'

run_expect_failure \
    "empty stdin runtime validation" \
    'printf "" | cargo run --quiet -p petit-tui -- --stdin' \
    'stdin is empty'

run_expect_failure \
    "benchmark run-count validation" \
    'cargo run --quiet -p petit-tui -- --benchmark --runs 0' \
    '--runs must be at least 1'

run_optional_model_smoke

printf '\n'
printf 'Summary: PASS=%s FAIL=%s SKIP=%s TOTAL=%s\n' \
    "$PASS_COUNT" "$FAIL_COUNT" "$SKIP_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
    exit 1
fi

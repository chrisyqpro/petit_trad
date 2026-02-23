#!/usr/bin/env sh
# scripts/eval.sh - translation regression harness for real local translations
#
# Usage:
#   ./scripts/eval.sh --model /path/to/model.gguf
#   ./scripts/eval.sh --model /path/to/model.gguf --fixtures eval/fixtures/smoke.tsv
#   ./scripts/eval.sh --model /path/to/model.gguf --features metal

set -eu

FIXTURES="eval/fixtures/smoke.tsv"
FEATURES="cpu-only"
MODEL_PATH=""
GPU_LAYERS="0"
CONTEXT_SIZE="256"
THREADS="1"

usage() {
    cat <<'EOF'
Usage: ./scripts/eval.sh --model <path> [options]

Options:
  --model <path>      Path to the GGUF model file (required)
  --fixtures <path>   Fixture TSV file (default: eval/fixtures/smoke.tsv)
  --features <value>  Cargo feature set (default: cpu-only)
  --gpu-layers <n>    GPU layers override (default: 0)
  --context-size <n>  Context size override (default: 256)
  --threads <n>       CPU threads override (default: 1)
  --help              Print this help and exit

Fixture format (TSV):
  case_id<TAB>source_lang<TAB>target_lang<TAB>input_text<TAB>expected_output
  - Lines starting with # are comments
  - Blank lines are ignored
  - input_text and expected_output must be single-line and contain no tabs
  - The harness rejects command-like/path-like/email-like fixture
    fields to avoid unsafe or sensitive test data in committed fixtures
EOF
}

die_usage() {
    echo "Error: $1" >&2
    echo >&2
    usage >&2
    exit 2
}

is_suspicious_fixture_field() {
    value=$1

    # Reject obvious shell/control patterns and local-path / secret-like forms.
    case "$value" in
        *'`'*|*'$('*|*'${'*|*';'*|*'&&'*|*'||'*|*'|'*|*'>'*|*'<'*)
            return 0
            ;;
        ./*|../*|/*|~/*)
            return 0
            ;;
        *://*|*@*.*)
            return 0
            ;;
        [A-Za-z]:\\*)
            return 0
            ;;
        -*|*' --'*)
            return 0
            ;;
    esac

    return 1
}

validate_fixture_safety() {
    case_id=$1
    field_name=$2
    value=$3

    if is_suspicious_fixture_field "$value"; then
        ERROR_COUNT=$((ERROR_COUNT + 1))
        echo "[ERROR] unsafe fixture $field_name for $case_id"
        echo "  use synthetic non-sensitive inputs"
        return 1
    fi

    return 0
}

while [ $# -gt 0 ]; do
    case "$1" in
        --model)
            [ $# -ge 2 ] || die_usage "Missing value for --model"
            MODEL_PATH=$2
            shift 2
            ;;
        --fixtures)
            [ $# -ge 2 ] || die_usage "Missing value for --fixtures"
            FIXTURES=$2
            shift 2
            ;;
        --features)
            [ $# -ge 2 ] || die_usage "Missing value for --features"
            FEATURES=$2
            shift 2
            ;;
        --gpu-layers)
            [ $# -ge 2 ] || die_usage "Missing value for --gpu-layers"
            GPU_LAYERS=$2
            shift 2
            ;;
        --context-size)
            [ $# -ge 2 ] || die_usage "Missing value for --context-size"
            CONTEXT_SIZE=$2
            shift 2
            ;;
        --threads)
            [ $# -ge 2 ] || die_usage "Missing value for --threads"
            THREADS=$2
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            die_usage "Unknown argument: $1"
            ;;
    esac
done

[ -n "$MODEL_PATH" ] || die_usage "--model is required"
[ -f "$MODEL_PATH" ] || die_usage "Model file not found: $MODEL_PATH"
[ -f "$FIXTURES" ] || die_usage "Fixture file not found: $FIXTURES"

TAB=$(printf '\t')
PASS_COUNT=0
FAIL_COUNT=0
ERROR_COUNT=0
CASE_COUNT=0

ERR_LOG=$(mktemp "${TMPDIR:-/tmp}/petit-eval-stderr.XXXXXX")
cleanup() {
    rm -f "$ERR_LOG"
}
trap cleanup EXIT INT TERM HUP

run_case() {
    case_id=$1
    source_lang=$2
    target_lang=$3
    input_text=$4
    expected_output=$5

    CASE_COUNT=$((CASE_COUNT + 1))

    # Clear the stderr capture file before each translation run.
    : >"$ERR_LOG"

    set +e
    actual_output=$(
        printf '%s' "$input_text" | \
            PETIT_TRAD_MODEL= \
            PETIT_TRAD_GPU_LAYERS= \
            PETIT_TRAD_CONTEXT_SIZE= \
            PETIT_TRAD_THREADS= \
            PETIT_TRAD_LOG_TO_FILE= \
            PETIT_TRAD_LOG_PATH= \
            PETIT_TRAD_SOURCE_LANG= \
            PETIT_TRAD_TARGET_LANG= \
            PETIT_TRAD_COMPACT_LANG= \
            PETIT_TRAD_NO_CONFIG= \
            cargo run -p petit-tui --features "$FEATURES" -- \
                --stdin \
                --no-config \
                --model "$MODEL_PATH" \
                --source-lang "$source_lang" \
                --target-lang "$target_lang" \
                --gpu-layers "$GPU_LAYERS" \
                --context-size "$CONTEXT_SIZE" \
                --threads "$THREADS" \
            2>"$ERR_LOG"
    )
    cmd_status=$?
    set -e

    if [ "$cmd_status" -ne 0 ]; then
        ERROR_COUNT=$((ERROR_COUNT + 1))
        echo "[ERROR] $case_id (translator command failed, exit $cmd_status)"
        if [ -s "$ERR_LOG" ]; then
            sed 's/^/  stderr: /' "$ERR_LOG"
        fi
        return
    fi

    if [ "$actual_output" = "$expected_output" ]; then
        PASS_COUNT=$((PASS_COUNT + 1))
        echo "[PASS] $case_id"
        return
    fi

    FAIL_COUNT=$((FAIL_COUNT + 1))
    echo "[FAIL] $case_id"
    echo "  expected: $expected_output"
    echo "  actual:   $actual_output"
}

case_id=''
source_lang=''
target_lang=''
input_text=''
expected_output=''
extra=''
while IFS="$TAB" read -r case_id source_lang target_lang input_text expected_output extra || \
    [ -n "${case_id}${source_lang}${target_lang}${input_text}${expected_output}${extra}" ]; do
    if [ -z "${source_lang}${target_lang}${input_text}${expected_output}${extra}" ]; then
        case "$case_id" in
            ''|'#'*)
                continue
                ;;
        esac
        ERROR_COUNT=$((ERROR_COUNT + 1))
        echo "[ERROR] malformed fixture row (need 5 tab-separated columns): $case_id"
        continue
    fi

    case "$case_id" in
        ''|'#'*)
            # Comments and blank lines must occupy the full line.
            if [ -n "${source_lang}${target_lang}${input_text}${expected_output}${extra}" ]; then
                ERROR_COUNT=$((ERROR_COUNT + 1))
                echo "[ERROR] malformed comment row in fixtures: $case_id"
            fi
            continue
            ;;
    esac

    if [ -n "${extra}" ]; then
        ERROR_COUNT=$((ERROR_COUNT + 1))
        echo "[ERROR] malformed fixture row (too many columns): $case_id"
        continue
    fi

    if [ -z "$source_lang" ] || [ -z "$target_lang" ]; then
        ERROR_COUNT=$((ERROR_COUNT + 1))
        echo "[ERROR] malformed fixture row (empty required metadata): $case_id"
        continue
    fi

    validate_fixture_safety "$case_id" "input_text" "$input_text" || continue
    validate_fixture_safety "$case_id" "expected_output" "$expected_output" || continue

    run_case "$case_id" "$source_lang" "$target_lang" "$input_text" "$expected_output"
done <"$FIXTURES"

echo "Summary: $PASS_COUNT passed, $FAIL_COUNT failed, $ERROR_COUNT errors"

if [ "$FAIL_COUNT" -gt 0 ] || [ "$ERROR_COUNT" -gt 0 ]; then
    exit 1
fi

exit 0

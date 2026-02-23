#!/usr/bin/env sh
# scripts/eval-capture.sh - generate translation fixture TSV from a curated corpus
#
# Usage:
#   ./scripts/eval-capture.sh --model /path/to/model.gguf
#   ./scripts/eval-capture.sh --model /path/to/model.gguf --features metal

set -eu

INPUT_TSV="eval/fixtures/smoke-inputs.tsv"
OUTPUT_TSV="eval/fixtures/smoke.tsv"
FEATURES="cpu-only"
MODEL_PATH=""
GPU_LAYERS="0"
CONTEXT_SIZE="256"
THREADS="1"

usage() {
    cat <<'EOF'
Usage: ./scripts/eval-capture.sh --model <path> [options]

Options:
  --model <path>      Path to the GGUF model file (required)
  --input <path>      Input corpus TSV (default: eval/fixtures/smoke-inputs.tsv)
  --output <path>     Output fixture TSV (default: eval/fixtures/smoke.tsv)
  --features <value>  Cargo feature set (default: cpu-only)
  --gpu-layers <n>    GPU layers override (default: 0)
  --context-size <n>  Context size override (default: 256)
  --threads <n>       CPU threads override (default: 1)
  --help              Print this help and exit

Input corpus format (TSV):
  case_id<TAB>source_lang<TAB>target_lang<TAB>input_text
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

TAB=$(printf '\t')
NL='
'

while [ $# -gt 0 ]; do
    case "$1" in
        --model)
            [ $# -ge 2 ] || die_usage "Missing value for --model"
            MODEL_PATH=$2
            shift 2
            ;;
        --input)
            [ $# -ge 2 ] || die_usage "Missing value for --input"
            INPUT_TSV=$2
            shift 2
            ;;
        --output)
            [ $# -ge 2 ] || die_usage "Missing value for --output"
            OUTPUT_TSV=$2
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
[ -f "$INPUT_TSV" ] || die_usage "Input corpus file not found: $INPUT_TSV"

OUT_DIR=$(dirname "$OUTPUT_TSV")
mkdir -p "$OUT_DIR"

TMP_OUT=$(mktemp "${TMPDIR:-/tmp}/petit-eval-capture.XXXXXX")
ERR_LOG=$(mktemp "${TMPDIR:-/tmp}/petit-eval-capture-stderr.XXXXXX")
cleanup() {
    rm -f "$TMP_OUT" "$ERR_LOG"
}
trap cleanup EXIT INT TERM HUP

{
    echo "# petit_trad translation eval fixtures (v1)"
    echo "# GENERATED FILE: captured from scripts/eval-capture.sh"
    echo "# Schema:"
    echo "# case_id<TAB>source_lang<TAB>target_lang<TAB>input_text<TAB>expected_output"
    echo "#"
    echo "# This file is for actual translation-behavior checks (simple phrases,"
    echo "# literals, localized formatting). Regenerate on the same model/backend"
    echo "# config when intentionally updating expectations."
} >"$TMP_OUT"

COUNT=0
case_id=''
source_lang=''
target_lang=''
input_text=''
extra=''
while IFS="$TAB" read -r case_id source_lang target_lang input_text extra || \
    [ -n "${case_id}${source_lang}${target_lang}${input_text}${extra}" ]; do
    if [ -z "${source_lang}${target_lang}${input_text}${extra}" ]; then
        case "$case_id" in
            ''|'#'*)
                continue
                ;;
        esac
        echo "Error: malformed corpus row (need 4 tab-separated columns): $case_id" >&2
        exit 1
    fi

    case "$case_id" in
        ''|'#'*)
            if [ -n "${source_lang}${target_lang}${input_text}${extra}" ]; then
                echo "Error: malformed comment row in corpus: $case_id" >&2
                exit 1
            fi
            continue
            ;;
    esac

    if [ -n "$extra" ]; then
        echo "Error: malformed corpus row (too many columns): $case_id" >&2
        exit 1
    fi

    if [ -z "$source_lang" ] || [ -z "$target_lang" ]; then
        echo "Error: malformed corpus row (empty language metadata): $case_id" >&2
        exit 1
    fi

    if is_suspicious_fixture_field "$input_text"; then
        echo "Error: unsafe corpus input_text for $case_id" >&2
        exit 1
    fi

    : >"$ERR_LOG"
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
    ) || {
        echo "Error: translation command failed for $case_id" >&2
        if [ -s "$ERR_LOG" ]; then
            sed 's/^/  stderr: /' "$ERR_LOG" >&2
        fi
        exit 1
    }

    case "$actual_output" in
        *"$TAB"*)
            echo "Error: output contains tab for $case_id; unsupported in TSV v1" >&2
            exit 1
            ;;
    esac
    case "$actual_output" in
        *"$NL"*)
            echo "Error: output contains newline for $case_id; unsupported in TSV v1" >&2
            exit 1
            ;;
    esac

    printf '%s\t%s\t%s\t%s\t%s\n' \
        "$case_id" "$source_lang" "$target_lang" "$input_text" "$actual_output" >>"$TMP_OUT"

    COUNT=$((COUNT + 1))
    echo "[CAPTURED] $case_id"
done <"$INPUT_TSV"

mv "$TMP_OUT" "$OUTPUT_TSV"
echo "Wrote $COUNT cases to $OUTPUT_TSV"

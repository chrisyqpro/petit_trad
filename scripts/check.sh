#!/usr/bin/env sh
# scripts/check.sh - canonical verification script for petit_trad
#
# Usage:
#   ./scripts/check.sh              # check-only (CI mode)
#   ./scripts/check.sh --fix        # auto-format then verify (pre-commit mode)
#   ./scripts/check.sh --features metal  # override default cpu-only features
#
# Runs in order: fmt, clippy, check test. Exits on first failure.

set -eu

FEATURES="cpu-only"
FIX=0

while [ $# -gt 0 ]; do
    case "$1" in
        --fix)
            FIX=1
            shift
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

echo "==> fmt"
if [ "$FIX" -eq 1 ]; then
    cargo fmt
else
    cargo fmt --check
fi

echo "==> clippy (features: $FEATURES)"
cargo clippy --workspace --features "$FEATURES" -- -D warnings

echo "==> check (features: $FEATURES)"
cargo check --workspace --features "$FEATURES"

echo "==> test (features: $FEATURES)"
cargo test --workspace --features "$FEATURES"

echo "==> all checks passed"

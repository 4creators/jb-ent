#!/bin/bash
# vendor-grammar.sh: Vendor a single tree-sitter grammar into internal/cbm/vendored/grammars/<name>/
# Usage: ./scripts/vendor-grammar.sh <repo_url> <name> [subdir] [hash]

set -euo pipefail

REPO_URL="$1"
NAME="$2"
SUBDIR="${3:-}"
HASH="${4:-}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
GRAMMAR_DIR="$PROJECT_DIR/internal/cbm/vendored/grammars/$NAME"
TMPDIR="$(mktemp -d)"

trap 'rm -rf "$TMPDIR"' EXIT

echo "Vendoring $NAME from $REPO_URL (Hash: ${HASH:-latest})..."

mkdir -p "$TMPDIR/repo"
cd "$TMPDIR/repo"
git init -q
git remote add origin "$REPO_URL"

if [ -n "$HASH" ] && [ "$HASH" != "UNKNOWN" ]; then
    git fetch --depth 1 origin "$HASH" 2>/dev/null || git fetch origin "$HASH" 2>/dev/null
    git checkout -q FETCH_HEAD
else
    git fetch --depth 1 origin HEAD 2>/dev/null
    git checkout -q FETCH_HEAD
fi

cd "$SCRIPT_DIR"

SRC_DIR="$TMPDIR/repo/src"
if [ -n "$SUBDIR" ]; then
    SRC_DIR="$TMPDIR/repo/$SUBDIR/src"
fi

if [ ! -f "$SRC_DIR/parser.c" ]; then
    echo "ERROR: $SRC_DIR/parser.c not found" >&2
    exit 1
fi

mkdir -p "$GRAMMAR_DIR/tree_sitter"
cp "$SRC_DIR/parser.c" "$GRAMMAR_DIR/"

if [ -f "$SRC_DIR/scanner.c" ]; then
    cp "$SRC_DIR/scanner.c" "$GRAMMAR_DIR/"
fi

if [ -d "$SRC_DIR/tree_sitter" ]; then
    cp "$SRC_DIR/tree_sitter/"*.h "$GRAMMAR_DIR/tree_sitter/" 2>/dev/null || true
fi

REPO_ROOT="$TMPDIR/repo"
if [ -n "$SUBDIR" ]; then
    if [ -f "$REPO_ROOT/$SUBDIR/LICENSE" ]; then
        cp "$REPO_ROOT/$SUBDIR/LICENSE" "$GRAMMAR_DIR/LICENSE"
    elif [ -f "$REPO_ROOT/LICENSE" ]; then
        cp "$REPO_ROOT/LICENSE" "$GRAMMAR_DIR/LICENSE"
    elif [ -f "$REPO_ROOT/LICENSE.md" ]; then
        cp "$REPO_ROOT/LICENSE.md" "$GRAMMAR_DIR/LICENSE"
    elif [ -f "$REPO_ROOT/COPYING" ]; then
        cp "$REPO_ROOT/COPYING" "$GRAMMAR_DIR/LICENSE"
    else
        echo "WARNING: No LICENSE file found for $NAME" >&2
    fi
else
    if [ -f "$REPO_ROOT/LICENSE" ]; then
        cp "$REPO_ROOT/LICENSE" "$GRAMMAR_DIR/LICENSE"
    elif [ -f "$REPO_ROOT/LICENSE.md" ]; then
        cp "$REPO_ROOT/LICENSE.md" "$GRAMMAR_DIR/LICENSE"
    elif [ -f "$REPO_ROOT/COPYING" ]; then
        cp "$REPO_ROOT/COPYING" "$GRAMMAR_DIR/LICENSE"
    else
        echo "WARNING: No LICENSE file found for $NAME" >&2
    fi
fi

echo "Vendored $NAME to $GRAMMAR_DIR"
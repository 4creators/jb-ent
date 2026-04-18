#!/bin/bash
# vendor-all-grammars.sh: Batch-restores all tree-sitter grammars from THIRD_PARTY.md
# Handles parser.c, scanner.c, and grammar.c artifacts with monorepo support.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
THIRD_PARTY_MD="$SCRIPT_DIR/../THIRD_PARTY.md"
VENDOR_SCRIPT="$SCRIPT_DIR/vendor-grammar.sh"

if [ ! -f "$THIRD_PARTY_MD" ]; then
    echo "ERROR: THIRD_PARTY.md not found at $THIRD_PARTY_MD" >&2
    exit 1
fi

echo "Scanning $THIRD_PARTY_MD for grammars..."

FAILED_LANGS=()

# Extract rows from the grammar table and process them
grep "^| [a-z0-9_]* " "$THIRD_PARTY_MD" | grep "tree-sitter" | while read -r line; do
    LANG=$(echo "$line" | cut -d'|' -f2 | tr -d ' ')
    HASH=$(echo "$line" | cut -d'|' -f3 | tr -d ' ')
    URL=$(echo "$line" | sed -n 's/.*(\(https:\/\/github.com\/[^)]*\)).*/\1/p')

    if [ -n "$URL" ] && [ "$HASH" != "SKIP" ] && [ "$HASH" != "ERROR" ]; then
        echo "----------------------------------------------------------------"
        echo "Processing $LANG: $URL (Hash: $HASH)"

        SUBDIR=""
        case "$LANG" in
            tsx|typescript) SUBDIR="typescript" ;;
            ocaml)          SUBDIR="grammars/ocaml" ;;
            php)            SUBDIR="php" ;;
            fsharp)         SUBDIR="fsharp" ;;
            markdown)       SUBDIR="tree-sitter-markdown" ;;
            xml)            SUBDIR="xml" ;;
        esac

        if ! bash "$VENDOR_SCRIPT" "$URL" "$LANG" "$SUBDIR" "$HASH"; then
            echo "ERROR: Failed to vendor $LANG" >&2
            FAILED_LANGS+=("$LANG")
        fi
    fi
done

if [ ${#FAILED_LANGS[@]} -ne 0 ]; then
    echo "----------------------------------------------------------------"
    echo "Grammar restoration completed with errors for: ${FAILED_LANGS[*]}"
    exit 1
fi

echo "Grammar restoration complete."
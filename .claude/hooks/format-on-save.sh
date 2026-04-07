#!/bin/bash
# Hook: PostToolUse — Auto-format despues de cada edicion

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.filePath // empty')

if [ -z "$FILE_PATH" ] || [ ! -f "$FILE_PATH" ]; then
  exit 0
fi

EXTENSION="${FILE_PATH##*.}"

case "$EXTENSION" in
  rs)
    if command -v rustfmt &> /dev/null; then
      rustfmt "$FILE_PATH" 2>/dev/null || true
    fi
    ;;
  sql)
    # No hay formatter estandar para SQL, skip
    ;;
  yml|yaml)
    # Validar YAML syntax
    if command -v python3 &> /dev/null; then
      python3 -c "import yaml; yaml.safe_load(open('$FILE_PATH'))" 2>/dev/null || true
    fi
    ;;
  toml)
    # Validar TOML syntax
    if command -v taplo &> /dev/null; then
      taplo fmt "$FILE_PATH" 2>/dev/null || true
    fi
    ;;
esac

exit 0

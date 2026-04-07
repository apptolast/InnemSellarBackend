#!/bin/bash
# Hook: PreToolUse — Bloquea escritura de secrets en codigo
# Recibe JSON en stdin con tool_input
# Exit code 0 = permite
# Exit code 2 = bloquea con feedback

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // .tool_input.filePath // empty')
CONTENT=$(echo "$INPUT" | jq -r '.tool_input.content // .tool_input.new_string // empty')

# Bloquear escritura en archivos sensibles
BLOCKED_PATTERNS=(".env" ".env." "secrets/" ".key" ".pem" "id_rsa" "credentials" "serviceAccountKey")
for pattern in "${BLOCKED_PATTERNS[@]}"; do
  if [[ "$FILE_PATH" == *"$pattern"* ]]; then
    echo "Escritura bloqueada: no se permite modificar archivos sensibles ($FILE_PATH)" >&2
    exit 2
  fi
done

# Detectar secrets en contenido
if [ -n "$CONTENT" ]; then
  # API keys, tokens, passwords hardcodeados
  if echo "$CONTENT" | grep -qiE '(api[_-]?key|secret[_-]?key|password|token|private[_-]?key)\s*[:=]\s*["\x27][A-Za-z0-9+/=_-]{20,}'; then
    echo "Posible secret detectado en el contenido. Usa variables de entorno en su lugar." >&2
    exit 2
  fi

  # Database URLs con credenciales
  if echo "$CONTENT" | grep -qiE 'postgres(ql)?://[^:]+:[^@]+@'; then
    echo "Database URL con credenciales detectada. Usa DATABASE_URL como variable de entorno." >&2
    exit 2
  fi
fi

exit 0

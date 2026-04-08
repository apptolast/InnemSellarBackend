#!/bin/bash
# Hook: TaskCompleted — Quality gate antes de marcar tarea como completa
# Exit code 0 = permite completion
# Exit code 2 = previene completion y envia feedback al agente

set -e

INPUT=$(cat)
CWD=$(echo "$INPUT" | jq -r '.cwd // "."')

cd "$CWD"

# Solo ejecutar checks si hay un proyecto Cargo
if [ ! -f "Cargo.toml" ]; then
  echo "No Cargo.toml encontrado, saltando quality gate Rust."
  exit 0
fi

# Verificar que compila
if ! cargo check 2>/dev/null; then
  echo "cargo check fallo. Corrige los errores de compilacion antes de completar la tarea." >&2
  exit 2
fi

# Verificar clippy (lints)
if ! cargo clippy -- -D warnings 2>/dev/null; then
  echo "cargo clippy encontro warnings. Corrige los warnings de clippy antes de completar." >&2
  exit 2
fi

# Verificar formato
if ! cargo fmt --check 2>/dev/null; then
  echo "cargo fmt --check fallo. Ejecuta 'cargo fmt' para formatear el codigo." >&2
  exit 2
fi

# Verificar que la documentacion compila (detecta links rotos, syntax errors en docs)
if ! cargo doc --no-deps 2>/dev/null; then
  echo "cargo doc fallo. Revisa que los doc comments (///) compilen correctamente." >&2
  exit 2
fi

# Ejecutar tests
if ! cargo test 2>/dev/null; then
  echo "cargo test fallo. Corrige los tests antes de completar la tarea." >&2
  exit 2
fi

echo "Quality gate passed: check, clippy, fmt, test"
exit 0

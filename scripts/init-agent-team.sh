#!/bin/bash
set -euo pipefail

# ============================================================================
# INEMSELLAR BACKEND — Agent Team Initializer
# Usage: ./scripts/init-agent-team.sh [path-to-technical-spec]
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SPEC_FILE="${1:-docs/technical-spec.md}"
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo -e "${BLUE}================================================${NC}"
echo -e "${BLUE}  InemSellar Backend — Agent Team Initializer    ${NC}"
echo -e "${BLUE}================================================${NC}"

# 1. Verificar prerequisitos
echo -e "\n${YELLOW}[1/5] Verificando prerequisitos...${NC}"

if ! command -v claude &> /dev/null; then
  echo -e "${RED}Claude Code no esta instalado.${NC}"
  echo -e "  Instala con: npm install -g @anthropic-ai/claude-code"
  exit 1
fi

CLAUDE_VERSION=$(claude --version 2>/dev/null || echo "unknown")
echo -e "${GREEN}  Claude Code: $CLAUDE_VERSION${NC}"

if [ ! -f "$PROJECT_DIR/$SPEC_FILE" ]; then
  echo -e "${YELLOW}  Documento tecnico no encontrado: $SPEC_FILE${NC}"
  echo -e "  El equipo usara schema.sql y CLAUDE.md como contexto."
else
  echo -e "${GREEN}  Spec: $SPEC_FILE${NC}"
fi

if [ ! -f "$PROJECT_DIR/schema.sql" ]; then
  echo -e "${RED}  schema.sql no encontrado. Es necesario para el equipo.${NC}"
  exit 1
fi
echo -e "${GREEN}  schema.sql encontrado${NC}"

# 2. Verificar configuracion
echo -e "\n${YELLOW}[2/5] Verificando configuracion...${NC}"

if [ ! -f "$PROJECT_DIR/.claude/settings.json" ]; then
  echo -e "${RED}  .claude/settings.json no encontrado.${NC}"
  exit 1
fi

if grep -q "CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS" "$PROJECT_DIR/.claude/settings.json"; then
  echo -e "${GREEN}  Agent Teams habilitado en settings.json${NC}"
else
  echo -e "${RED}  CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS no encontrado en settings.json${NC}"
  exit 1
fi

# 3. Verificar agentes definidos
echo -e "\n${YELLOW}[3/5] Verificando definiciones de agentes...${NC}"

AGENTS_DIR="$PROJECT_DIR/.claude/agents"
AGENT_COUNT=$(ls -1 "$AGENTS_DIR"/*.md 2>/dev/null | wc -l)
echo -e "${GREEN}  $AGENT_COUNT agentes definidos en .claude/agents/:${NC}"
for agent_file in "$AGENTS_DIR"/*.md; do
  agent_name=$(basename "$agent_file" .md)
  echo -e "    - $agent_name"
done

# 4. Hacer hooks ejecutables
echo -e "\n${YELLOW}[4/5] Configurando hooks...${NC}"

for hook_file in "$PROJECT_DIR"/.claude/hooks/*.sh; do
  if [ -f "$hook_file" ]; then
    chmod +x "$hook_file"
    echo -e "${GREEN}  chmod +x $(basename "$hook_file")${NC}"
  fi
done

# 5. Verificar git
echo -e "\n${YELLOW}[5/5] Verificando git...${NC}"

cd "$PROJECT_DIR"
if [ ! -d ".git" ]; then
  echo -e "${YELLOW}  No hay repositorio git. Inicializando...${NC}"
  git init
fi

# Asegurar .gitignore
GITIGNORE_ENTRIES=(
  "CLAUDE.local.md"
  ".claude/settings.local.json"
  ".env"
  ".env.*"
  "!.env.example"
  "target/"
  "*.pem"
  "*.key"
)

touch .gitignore
for entry in "${GITIGNORE_ENTRIES[@]}"; do
  if ! grep -qF "$entry" .gitignore 2>/dev/null; then
    echo "$entry" >> .gitignore
  fi
done
echo -e "${GREEN}  Git configurado${NC}"

# Lanzar Claude Code
echo -e "\n${BLUE}================================================${NC}"
echo -e "${GREEN}  Todo listo. Lanzando Claude Code...${NC}"
echo -e "${BLUE}================================================${NC}"
echo ""
echo -e "${YELLOW}  Controles del equipo:${NC}"
echo -e "    Shift+Tab     → Activar Delegate Mode (recomendado)"
echo -e "    Shift+Down    → Navegar entre teammates"
echo -e "    Ctrl+T        → Ver task list"
echo -e "    /project:init-team  → Inicializar equipo automaticamente"
echo -e "    /project:status-report → Ver estado del equipo"
echo ""

# Preparar el prompt de arranque
SPEC_CONTENT=""
if [ -f "$PROJECT_DIR/$SPEC_FILE" ]; then
  SPEC_CONTENT=$(cat "$PROJECT_DIR/$SPEC_FILE")
fi

cd "$PROJECT_DIR"
claude --append-system-prompt "
CONTEXTO: Eres el Team Lead del proyecto InemSellar Backend.
Tu rol es UNICAMENTE coordinar el equipo. Activa Delegate Mode (Shift+Tab).

PROYECTO: Backend Rust + PostgreSQL para la app InemSellar (ayuda a desempleados SEPE).
El schema.sql ya esta definido con 17 tablas.

DOCUMENTO TECNICO:
$SPEC_CONTENT

INSTRUCCIONES:
1. Lee CLAUDE.md y schema.sql para entender el proyecto
2. Crea un agent team con TeamCreate
3. Descomponer en tareas con TaskCreate siguiendo 5 waves:
   Wave 1: Arquitectura (architect con Opus)
   Wave 2: Implementacion (rust-dev + devops-engineer + dba-engineer con Sonnet)
   Wave 3: Testing (qa-engineer con Sonnet)
   Wave 4: Review (code-reviewer + security-reviewer)
   Wave 5: Mentoring (mentor con Opus)
4. Spawnar teammates usando las definiciones de .claude/agents/
5. Coordinar waves respetando dependencias entre tareas
6. Sintetizar resultados y resolver conflictos

REGLA: Cada teammate recibe contexto completo en su prompt de spawn
porque NO heredan tu historial de conversacion.
"

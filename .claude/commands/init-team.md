---
description: Inicializa un equipo de agentes basandose en el spec tecnico
allowed-tools: Read, Bash, Write
---
Lee el documento tecnico en docs/technical-spec.md y el CLAUDE.md del proyecto,
luego crea un agent team completo para implementar el backend.

Pasos:
1. Analizar el spec tecnico y el schema.sql
2. Determinar que roles son necesarios para este proyecto
3. Crear el equipo con TeamCreate
4. Definir todas las tareas con dependencias usando TaskCreate siguiendo las waves:
   - Wave 1: Arquitectura (architect)
   - Wave 2: Implementacion paralela (rust-dev + devops-engineer + dba-engineer)
   - Wave 3: Testing (qa-engineer)
   - Wave 4: Review (code-reviewer + security-reviewer)
   - Wave 5: Documentacion y enseñanza (mentor)
5. Spawnar teammates con prompts detallados usando las definiciones en .claude/agents/
6. Activar Delegate Mode (Shift+Tab)

Roles disponibles en .claude/agents/:
architect, rust-dev, devops-engineer, dba-engineer, security-reviewer,
qa-engineer, code-reviewer, mentor

Modelos recomendados:
- Opus: architect, security-reviewer, mentor (razonamiento profundo)
- Sonnet: rust-dev, devops-engineer, dba-engineer, qa-engineer, code-reviewer

Usar $ARGUMENTS para instrucciones adicionales del usuario.

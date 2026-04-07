---
description: Genera un reporte de estado del equipo de agentes
allowed-tools: Read, Bash, Grep
---
Genera un reporte completo del estado actual del equipo:

1. Lista todas las tareas con TaskList y su estado (pending, in_progress, completed)
2. Identifica tareas bloqueadas y por que
3. Resume el progreso por cada teammate
4. Identifica risks y blockers
5. Estima progreso general (%)

Formato de salida: tabla markdown con status por tarea y resumen ejecutivo.

Ejemplo:
```
## Estado del equipo — {{fecha}}

| # | Tarea | Asignado | Estado | Bloqueada por |
|---|-------|----------|--------|---------------|
| 1 | Disenar arquitectura | architect | completed | — |
| 2 | Contratos API | architect | in_progress | — |
| 3 | Setup Cargo | rust-dev | pending | #1 |

### Resumen
- Progreso general: 15% (2/15 tareas completadas)
- Teammates activos: 3/8
- Blockers: Ninguno
- Siguiente wave: Wave 2 (implementacion) comenzara cuando Wave 1 complete
```

# InemSellar Backend — Deployment & Infrastructure

## Descripcion del proyecto
InemSellar es una app Flutter de ayuda a desempleados en Espana (SEPE/INEM).
Este repositorio contiene el backend en Rust + PostgreSQL y toda la infraestructura
de despliegue: Docker, CI/CD, Nginx, migraciones, y scripts de operaciones.

## Stack tecnologico
- **Backend**: Rust (framework por decidir: Axum o Actix-web)
- **Base de datos**: PostgreSQL 16+
- **ORM/Queries**: SQLx (compile-time checked queries)
- **Autenticacion**: JWT (access + refresh tokens)
- **Contenedores**: Docker + docker-compose
- **Proxy reverso**: Nginx
- **CI/CD**: GitHub Actions
- **Servidor**: VPS Linux (AppToLastServer)

## Comandos esenciales
- Build: `cargo build --release`
- Tests: `cargo test`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --check`
- Type check: `cargo check`
- Docker build: `docker compose build`
- Docker up: `docker compose up -d`
- Migraciones: `sqlx migrate run`
- Seed: `cargo run --bin seed`

## Estructura del proyecto
```
InnemBackendDespliegue/
├── src/                    # Codigo fuente Rust
│   ├── main.rs
│   ├── config/             # Configuracion (env vars, settings)
│   ├── routes/             # Handlers de endpoints
│   ├── models/             # Structs que mapean a tablas PostgreSQL
│   ├── services/           # Logica de negocio
│   └── middleware/         # Auth, logging, CORS
├── migrations/             # Migraciones SQL (SQLx)
├── tests/                  # Tests de integracion
├── scripts/                # Scripts de operaciones
├── nginx/                  # Config Nginx
├── .github/workflows/      # CI/CD pipelines
├── Dockerfile
├── docker-compose.yml
├── Cargo.toml
├── schema.sql              # Esquema PostgreSQL completo (referencia)
└── docs/                   # Documentacion tecnica
```

## Referencia al esquema de base de datos
El archivo `schema.sql` contiene el esquema completo con 17 tablas:
- Geografía: comunidades_autonomas, provincias, oficinas_sepe
- Auth: usuarios, proveedores_autenticacion, tokens_refresco
- Contenido: ofertas_empleo, consejos, cursos
- Relaciones N:M: ofertas_provincias, consejos_provincias, cursos_provincias
- Interacciones: votos (polimorfica), reportes (polimorfica)
- Sistema: configuracion_aplicacion, prestaciones

@schema.sql

## Convenciones de codigo

### Rust
- snake_case para funciones, variables, modulos
- PascalCase para structs y enums
- Clippy clean: `cargo clippy -- -D warnings` debe pasar sin errores
- Documentacion con `///` para funciones publicas
- Error handling con `thiserror` para errores propios, `anyhow` para propagacion
- Async con tokio runtime

### SQL / Migraciones
- Nombres en espanol con snake_case (consistente con schema.sql)
- Cada migracion es un archivo .sql en migrations/
- Migraciones reversibles cuando sea posible (up + down)

### Docker
- Multi-stage builds (builder + runtime)
- Imagen base: rust:slim para builder, debian:bookworm-slim para runtime
- No secrets en imagenes
- Health checks en cada servicio

### Git
- Commits atomicos con mensajes descriptivos en espanol
- Una feature por branch
- No force push a main

## Reglas para Agent Teams

### Ownership de archivos (CRITICO — dos agentes NUNCA editan el mismo archivo)
- **architect**: docs/architecture.md, docs/api-contracts.md
- **rust-dev**: src/**, Cargo.toml, Cargo.lock
- **devops-engineer**: Dockerfile, docker-compose.yml, .github/**, nginx/**, scripts/deploy*
- **dba-engineer**: migrations/**, scripts/seed*, scripts/backup*
- **qa-engineer**: tests/**
- **code-reviewer**: SOLO LECTURA
- **security-reviewer**: SOLO LECTURA
- **mentor**: SOLO LECTURA

### Sizing de tareas
- Cada tarea completable en 5-15 minutos
- Criterios de aceptacion claros en cada task description
- Target: 5-6 tasks por teammate
- Cada task = un commit atomico descriptivo

### Quality gates
- Todo codigo Rust debe pasar: `cargo clippy -- -D warnings && cargo fmt --check && cargo test`
- Code reviewer firma antes de merge
- Security checks obligatorios antes de PR

### Comunicacion entre agentes
- SendMessage para coordinar interfaces/contratos entre agentes
- Team Lead sintetiza y resuelve conflictos
- Mensajes directos para dependencias puntuales (no broadcast)
- Broadcasts solo para anuncios que afectan a todo el equipo

### Compaction
Cuando se compacte el contexto, preservar siempre:
- Lista completa de archivos modificados
- Estado actual de todas las tareas
- Comandos de build y test
- Decisiones de arquitectura tomadas
- Referencia al schema.sql

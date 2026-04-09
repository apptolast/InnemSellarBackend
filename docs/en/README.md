# InemSellar Backend — English Documentation

## Project Description

InemSellar is a Flutter app that helps unemployed people in Spain (SEPE/INEM).
This backend is written in Rust using the Salvo framework and PostgreSQL 16,
with SeaORM as the async ORM.

---

## Rust Language Features Used

### Ownership and Borrowing
Rust guarantees memory safety at compile time with no garbage collector.
The ownership system ensures each value has exactly one owner, and borrowing
allows temporary references without transferring ownership. Used here to pass
configuration and database connections without unnecessary cloning.

### Traits as Contracts (Interfaces)
Traits in Rust are equivalent to interfaces in other languages. This project uses
the **repository pattern**: each repository defines a trait with available operations,
and the concrete implementation implements it. This enables:
- Decoupling business logic from database implementation
- Easier testing with mock implementations
- Dependency Inversion Principle (SOLID)

Example: `AuthRepo` is a trait, `SeaAuthRepo` implements it using SeaORM.

### Async/Await with Tokio
The entire server is fully asynchronous. Tokio is Rust's async runtime:
- `async fn` defines functions that can be suspended without blocking an OS thread
- `.await` yields control to the scheduler while waiting for I/O (database, network)
- This allows handling thousands of concurrent connections with few OS threads

### Error Handling with thiserror and anyhow
- `thiserror`: defines custom errors with descriptive messages using `#[derive(Error)]`
- `anyhow`: propagates errors of any type up the call stack
- Pattern: inner layers use `thiserror` for typed errors; handlers use `anyhow` for
  simplified propagation.

### Lifetimes
Lifetimes ensure references do not outlive the data they point to.
In this project they are used mostly implicitly (lifetime elision), but appear
explicitly in some Salvo middleware contexts.

### Generics and Type-State
Generics allow reusable code with no runtime cost. SeaORM uses generics extensively
so queries are typed at compile time, catching schema errors before execution.

---

## Salvo Framework

Salvo is an async web framework for Rust. Features used:

- **affix-state**: dependency injection. Attaches shared state (DB connection,
  config) to the router and makes it accessible from any handler.
- **oapi**: automatic OpenAPI/Swagger documentation generation from handlers.
- **Router**: defines API routes with HTTP methods.
- **Handler/Endpoint**: async function that processes an HTTP request. `#[endpoint]`
  additionally generates OpenAPI metadata automatically.
- **Middleware (hoop)**: interceptors that run before/after handlers.
  Used for JWT authentication.

---

## Architecture

### Repository Pattern

```
Handler (HTTP) -> Service (business logic) -> Repository (data access) -> PostgreSQL
```

- Handlers only validate input and call the service
- Services contain business logic
- Repositories abstract all SQL queries through traits
- SeaORM generates typed queries from models

### JWT Authentication
- Access token: short-lived (15 min). Sent in `Authorization: Bearer <token>` header
- Refresh token: long-lived (30 days), stored in DB (`tokens_refresco` table), allows
  renewing the access token without re-authenticating
- Refresh token hash stored with SHA-256 (not plaintext) for security
- Passwords hashed with Argon2id (Password Hashing Competition winner)
- Implemented with the `jsonwebtoken` crate

### SeaORM
Async ORM on top of SQLx. Features:
- Typed entities mapping to PostgreSQL tables
- Rust query builder (no raw SQL in most cases)
- Support for UUIDs, timestamps, PostgreSQL arrays, JSONB

### OpenAPI/Swagger
- Automatic documentation generated from handlers with `#[endpoint]`
- Interactive Swagger UI at `/swagger-ui`
- Exportable JSON spec at `/api-doc/openapi.json` (importable in Postman)
- JWT security scheme integrated with Authorize button

---

## API Endpoints

### Authentication (`/api/v1/auth`)
- `POST /registro` — Register new user
- `POST /login` — Login with email/password, returns access + refresh tokens
- `POST /refrescar` — Renew access token using refresh token
- `POST /logout` — Revoke refresh token (requires auth)

### Job Offers (`/api/v1/ofertas`)
- `GET /` — List active offers (paginated, filter by province)
- `GET /{id}` — Offer detail
- `POST /` — Create offer (requires auth)
- `PUT /{id}` — Edit offer (requires being the author)
- `DELETE /{id}` — Delete offer (requires being the author)

### Geography (`/api/v1`)
- `GET /comunidades` — List autonomous communities
- `GET /comunidades/{id}` — Community detail
- `GET /provincias` — List provinces (optional community filter)
- `GET /provincias/{id}` — Province detail
- `GET /provincias/{id}/oficina` — SEPE office for the province

### System
- `GET /` — Health check (returns 200 OK)
- `GET /swagger-ui` — Interactive API documentation
- `GET /api-doc/openapi.json` — OpenAPI 3.1 spec

---

## Project Structure

```
inem-sellar-backend/src/
├── main.rs              — Entry point, configures Salvo + OpenAPI and starts the server
├── config/              — Environment variable loading with dotenvy/envy
├── db/                  — SeaORM connection pool to PostgreSQL
├── errors/              — Custom error types with thiserror + EndpointOutRegister
├── handlers/            — HTTP functions with #[endpoint] (Auth, Geography, Offers)
├── middleware/          — JWT auth (verifies Bearer token, injects user_id)
├── models/              — Structs mapping to PostgreSQL tables (17 SeaORM entities)
├── repositories/        — Traits + data access implementations
├── routes/              — Route tree definition with Salvo Router
└── services/            — Business logic (AuthService: JWT, Argon2, refresh tokens)
```

---

## Required Environment Variables

| Variable                  | Description                                  | Example                          |
|---------------------------|----------------------------------------------|----------------------------------|
| `DATABASE_URL`            | PostgreSQL connection URL                    | See K8s secret                   |
| `JWT_SECRET`              | Secret key for signing JWT tokens            | Random 48+ byte string           |
| `JWT_EXPIRACION_MINUTOS`  | Access token duration in minutes             | `15`                             |
| `SERVER_ADDR`             | Public address (for logs)                    | `0.0.0.0:8080`                   |
| `PORT_ADDR`               | Socket binding address                       | `0.0.0.0:8080`                   |
| `RUST_LOG`                | Log level (trace/debug/info/warn/error)      | `info`                           |

---

## Generated API Reference

The complete technical documentation generated by `cargo doc` is available at
`docs/api-reference/inem_sellar_backend/index.html`.

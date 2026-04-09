# InemSellar Backend — Documentacion en Espanol

## Descripcion del proyecto

InemSellar es una aplicacion Flutter de ayuda a desempleados en Espana (SEPE/INEM).
Este backend esta escrito en Rust usando el framework Salvo y PostgreSQL 16 como base
de datos, con SeaORM como ORM asincrono.

---

## Caracteristicas del lenguaje Rust utilizadas

### Ownership y Borrowing
Rust garantiza seguridad de memoria en tiempo de compilacion sin garbage collector.
El sistema de ownership hace que cada valor tenga exactamente un propietario, y el
borrowing permite referencias temporales sin transferir la propiedad. En este proyecto
se usa para pasar configuracion y conexiones de base de datos sin clonaciones innecesarias.

### Traits como contratos (interfaces)
Los traits en Rust son equivalentes a las interfaces en otros lenguajes. En este proyecto
se usa el **repository pattern**: cada repositorio define un trait con las operaciones
disponibles, y la implementacion concreta lo implementa. Esto permite:
- Desacoplar la logica de negocio de la implementacion de base de datos
- Facilitar el testing con implementaciones mock
- Seguir el principio de inversion de dependencias (SOLID)

Ejemplo: `AuthRepo` es un trait, `SeaAuthRepo` lo implementa usando SeaORM.

### Async/Await con Tokio
Todo el servidor es completamente asincrono. Tokio es el runtime async de Rust. La clave:
- `async fn` define funciones que pueden suspenderse sin bloquear el hilo del SO
- `.await` cede el control al scheduler mientras espera I/O (base de datos, red)
- Esto permite manejar miles de conexiones concurrentes con pocos hilos del SO

### Error Handling con thiserror y anyhow
- `thiserror`: define errores propios con mensajes descriptivos usando `#[derive(Error)]`
- `anyhow`: propaga errores de cualquier tipo hacia arriba en la pila de llamadas
- Patron: las capas internas usan `thiserror` para errores tipados; los handlers usan
  `anyhow` para simplificar la propagacion.

### Lifetimes
Los lifetimes garantizan que las referencias no outliven los datos a los que apuntan.
En este proyecto se usan principalmente de forma implicita (lifetime elision), pero
aparecen explicitamente en algunos contextos de middleware de Salvo.

### Genericos y tipo-estado
Los genericos permiten codigo reutilizable sin coste en runtime. SeaORM usa genericos
extensivamente para que las queries sean tipadas en tiempo de compilacion, detectando
errores de esquema antes de ejecutar.

---

## Framework Salvo

Salvo es un framework web asincrono para Rust. Caracteristicas usadas:

- **affix-state**: inyeccion de dependencias. Permite adjuntar estado compartido
  (conexion a BD, configuracion) al router y acceder desde cualquier handler.
- **oapi**: generacion automatica de documentacion OpenAPI/Swagger desde los handlers.
- **Router**: define las rutas de la API con metodos HTTP.
- **Handler/Endpoint**: funcion asincrona que procesa una peticion HTTP. `#[endpoint]`
  ademas genera metadata OpenAPI automaticamente.
- **Middleware (hoop)**: interceptores que se ejecutan antes/despues de los handlers.
  Se usan para autenticacion JWT.

---

## Arquitectura

### Repository Pattern

```
Handler (HTTP) -> Service (logica de negocio) -> Repository (acceso a datos) -> PostgreSQL
```

- Los handlers solo validan entrada y llaman al servicio
- Los servicios contienen la logica de negocio
- Los repositorios abstraen todas las queries SQL a traves de traits
- SeaORM genera queries tipadas a partir de los modelos

### Autenticacion JWT
- Access token: vida corta (15 min). Se envia en el header `Authorization: Bearer <token>`
- Refresh token: vida larga (30 dias), guardado en BD (tabla `tokens_refresco`), permite
  renovar el access token sin re-autenticarse
- El hash del refresh token se almacena con SHA-256 (no el token en claro) para seguridad
- Contrasenas hasheadas con Argon2id (ganador de la Password Hashing Competition)
- Implementado con la crate `jsonwebtoken`

### SeaORM
ORM asincrono sobre SQLx. Caracteristicas:
- Entidades tipadas que mapean a tablas PostgreSQL
- Query builder en Rust (sin SQL crudo en la mayoria de casos)
- Soporte para UUIDs, timestamps, arrays de PostgreSQL, JSONB

### OpenAPI/Swagger
- Documentacion automatica generada desde los handlers con `#[endpoint]`
- Swagger UI interactivo en `/swagger-ui`
- Spec JSON exportable en `/api-doc/openapi.json` (importable en Postman)
- Esquema de seguridad JWT integrado con boton Authorize

---

## Endpoints de la API

### Autenticacion (`/api/v1/auth`)
- `POST /registro` — Registrar nuevo usuario
- `POST /login` — Login con email/password, devuelve access + refresh tokens
- `POST /refrescar` — Renovar access token usando refresh token
- `POST /logout` — Revocar refresh token (requiere auth)

### Ofertas de empleo (`/api/v1/ofertas`)
- `GET /` — Listar ofertas activas (paginado, filtro por provincia)
- `GET /{id}` — Detalle de una oferta
- `POST /` — Crear oferta (requiere auth)
- `PUT /{id}` — Editar oferta (requiere ser autor)
- `DELETE /{id}` — Eliminar oferta (requiere ser autor)

### Geografia (`/api/v1`)
- `GET /comunidades` — Listar comunidades autonomas
- `GET /comunidades/{id}` — Detalle de comunidad
- `GET /provincias` — Listar provincias (filtro opcional por comunidad)
- `GET /provincias/{id}` — Detalle de provincia
- `GET /provincias/{id}/oficina` — Oficina SEPE de la provincia

### Sistema
- `GET /` — Health check (devuelve 200 OK)
- `GET /swagger-ui` — Documentacion interactiva
- `GET /api-doc/openapi.json` — Spec OpenAPI 3.1

---

## Estructura del proyecto

```
inem-sellar-backend/src/
├── main.rs              — Punto de entrada, configura Salvo + OpenAPI y arranca el servidor
├── config/              — Carga de variables de entorno con dotenvy/envy
├── db/                  — Pool de conexion SeaORM a PostgreSQL
├── errors/              — Tipos de error propios con thiserror + EndpointOutRegister
├── handlers/            — Funciones HTTP con #[endpoint] (Auth, Geografia, Ofertas)
├── middleware/          — Auth JWT (verifica Bearer token, inyecta id_usuario)
├── models/              — Structs que mapean a tablas PostgreSQL (17 entidades SeaORM)
├── repositories/        — Traits + implementaciones de acceso a datos
├── routes/              — Definicion del arbol de rutas con Salvo Router
└── services/            — Logica de negocio (AuthService: JWT, Argon2, refresh tokens)
```

---

## Variables de entorno requeridas

| Variable                  | Descripcion                                  | Ejemplo                       |
|---------------------------|----------------------------------------------|-------------------------------|
| `DATABASE_URL`            | URL de conexion a PostgreSQL                 | Ver secret de K8s             |
| `JWT_SECRET`              | Clave secreta para firmar tokens JWT         | Cadena aleatoria de 48+ bytes |
| `JWT_EXPIRACION_MINUTOS`  | Duracion del access token en minutos         | `15`                          |
| `SERVER_ADDR`             | Direccion publica (para logs)                | `0.0.0.0:8080`                |
| `PORT_ADDR`               | Direccion de binding del socket TCP          | `0.0.0.0:8080`                |
| `RUST_LOG`                | Nivel de logs (trace/debug/info/warn/error)  | `info`                        |

---

## Referencia API generada por rustdoc

La documentacion tecnica completa generada por `cargo doc` esta disponible en
`docs/api-reference/inem_sellar_backend/index.html`.

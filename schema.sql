-- ============================================================
-- InemSellar — Esquema PostgreSQL completo
-- 17 tablas, 6 dominios
-- Generado: 2026-04-06
-- ============================================================
--
-- Cambios respecto a version anterior:
--   1. Votos: INTEGER (1=upvote, -1=downvote, 0=sin voto) en vez de ENUM
--   2. Trigger automatico: actualiza contadores upvotes/downvotes en las
--      tablas de contenido cuando se inserta/actualiza/elimina un voto
--   3. Sin abreviaturas en nombres de tablas y campos
--   4. Todos los campos nullable excepto PKs y FKs
--   5. Cursos: campos adicionales para cursos comunitarios
-- ============================================================


-- ============================================================
-- ENUMS
-- ============================================================

-- Origen del contenido (solo para cursos: oficial + comunidad)
CREATE TYPE origen_contenido AS ENUM ('oficial', 'comunidad');

-- Estado de moderacion (para todo contenido comunitario)
CREATE TYPE estado_moderacion AS ENUM ('pendiente', 'aprobado', 'rechazado', 'en_revision');

-- Tipo de contenido (para tablas polimorficas: votos y reportes)
CREATE TYPE tipo_contenido AS ENUM ('oferta', 'consejo', 'curso');

-- Motivo de reporte
CREATE TYPE motivo_reporte AS ENUM ('spam', 'inapropiado', 'desactualizado', 'incorrecto', 'duplicado', 'otro');

-- Estado de reporte
CREATE TYPE estado_reporte AS ENUM ('pendiente', 'aceptado', 'rechazado');


-- ============================================================
-- FUNCIONES REUTILIZABLES
-- ============================================================

-- Actualiza automaticamente el campo actualizado_en en cada UPDATE
CREATE OR REPLACE FUNCTION set_actualizado_en() RETURNS trigger AS $$
BEGIN
    NEW.actualizado_en = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Actualiza automaticamente los contadores cantidad_upvotes / cantidad_downvotes
-- en la tabla de contenido correspondiente (ofertas_empleo, consejos, cursos)
-- cuando se inserta, actualiza o elimina un voto.
--
-- tipo_voto: 1 = upvote, -1 = downvote, 0 = sin voto (voto retirado)
--
-- La tabla votos es polimorfica: el campo tipo_contenido indica a que tabla
-- apunta id_contenido ('oferta' -> ofertas_empleo, 'consejo' -> consejos,
-- 'curso' -> cursos). No hay FK directa en id_contenido porque PostgreSQL
-- no soporta FK condicionales a multiples tablas.
CREATE OR REPLACE FUNCTION actualizar_contadores_votos() RETURNS trigger AS $$
DECLARE
    nombre_tabla TEXT;
    contenido_id UUID;
BEGIN
    -- Determinar tabla destino y id del contenido
    IF (TG_OP = 'DELETE') THEN
        contenido_id := OLD.id_contenido;
        nombre_tabla := CASE OLD.tipo_contenido
            WHEN 'oferta'  THEN 'ofertas_empleo'
            WHEN 'consejo' THEN 'consejos'
            WHEN 'curso'   THEN 'cursos'
        END;
    ELSE
        contenido_id := NEW.id_contenido;
        nombre_tabla := CASE NEW.tipo_contenido
            WHEN 'oferta'  THEN 'ofertas_empleo'
            WHEN 'consejo' THEN 'consejos'
            WHEN 'curso'   THEN 'cursos'
        END;
    END IF;

    IF (TG_OP = 'INSERT') THEN
        IF (NEW.tipo_voto = 1) THEN
            EXECUTE format(
                'UPDATE %I SET cantidad_upvotes = COALESCE(cantidad_upvotes, 0) + 1 WHERE id = $1',
                nombre_tabla
            ) USING contenido_id;
        ELSIF (NEW.tipo_voto = -1) THEN
            EXECUTE format(
                'UPDATE %I SET cantidad_downvotes = COALESCE(cantidad_downvotes, 0) + 1 WHERE id = $1',
                nombre_tabla
            ) USING contenido_id;
        END IF;
        RETURN NEW;

    ELSIF (TG_OP = 'UPDATE') THEN
        IF (OLD.tipo_voto IS DISTINCT FROM NEW.tipo_voto) THEN
            -- Restar voto anterior
            IF (OLD.tipo_voto = 1) THEN
                EXECUTE format(
                    'UPDATE %I SET cantidad_upvotes = COALESCE(cantidad_upvotes, 0) - 1 WHERE id = $1',
                    nombre_tabla
                ) USING contenido_id;
            ELSIF (OLD.tipo_voto = -1) THEN
                EXECUTE format(
                    'UPDATE %I SET cantidad_downvotes = COALESCE(cantidad_downvotes, 0) - 1 WHERE id = $1',
                    nombre_tabla
                ) USING contenido_id;
            END IF;
            -- Sumar voto nuevo
            IF (NEW.tipo_voto = 1) THEN
                EXECUTE format(
                    'UPDATE %I SET cantidad_upvotes = COALESCE(cantidad_upvotes, 0) + 1 WHERE id = $1',
                    nombre_tabla
                ) USING contenido_id;
            ELSIF (NEW.tipo_voto = -1) THEN
                EXECUTE format(
                    'UPDATE %I SET cantidad_downvotes = COALESCE(cantidad_downvotes, 0) + 1 WHERE id = $1',
                    nombre_tabla
                ) USING contenido_id;
            END IF;
        END IF;
        RETURN NEW;

    ELSIF (TG_OP = 'DELETE') THEN
        IF (OLD.tipo_voto = 1) THEN
            EXECUTE format(
                'UPDATE %I SET cantidad_upvotes = COALESCE(cantidad_upvotes, 0) - 1 WHERE id = $1',
                nombre_tabla
            ) USING contenido_id;
        ELSIF (OLD.tipo_voto = -1) THEN
            EXECUTE format(
                'UPDATE %I SET cantidad_downvotes = COALESCE(cantidad_downvotes, 0) - 1 WHERE id = $1',
                nombre_tabla
            ) USING contenido_id;
        END IF;
        RETURN OLD;
    END IF;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;


-- ============================================================
-- DOMINIO 1: GEOGRAFIA
-- ============================================================

-- 19 registros: 17 CCAA + Ceuta + Melilla
CREATE TABLE comunidades_autonomas (
    id                      SERIAL PRIMARY KEY,
    nombre                  TEXT UNIQUE,
    nombre_servicio_empleo  TEXT,              -- "SAE", "SOC", "Lanbide"...
    web_servicio_empleo     TEXT,
    url_sellado             TEXT,              -- enlace sellado/renovacion demanda
    creado_en               TIMESTAMPTZ DEFAULT now(),
    actualizado_en          TIMESTAMPTZ DEFAULT now()
);

-- 52 registros: provincias espanolas (codigos INE)
CREATE TABLE provincias (
    id              INTEGER PRIMARY KEY,      -- 1-52, codigo INE
    nombre          TEXT,
    id_comunidad    INTEGER NOT NULL REFERENCES comunidades_autonomas(id),
    logo_asset      TEXT,
    creado_en       TIMESTAMPTZ DEFAULT now(),
    actualizado_en  TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_provincias_comunidad ON provincias (id_comunidad);

-- 52 registros: datos SEPE por provincia
CREATE TABLE oficinas_sepe (
    id              SERIAL PRIMARY KEY,
    id_provincia    INTEGER NOT NULL UNIQUE REFERENCES provincias(id),
    telefono        TEXT,
    web             TEXT,
    url_cursos      TEXT,                     -- enlace catalogo cursos provincial
    url_orientacion TEXT,                     -- enlace orientacion provincial
    creado_en       TIMESTAMPTZ DEFAULT now(),
    actualizado_en  TIMESTAMPTZ DEFAULT now()
);


-- ============================================================
-- DOMINIO 2: AUTENTICACION Y USUARIOS
-- ============================================================

CREATE TABLE usuarios (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email             TEXT UNIQUE,
    hash_contrasena   TEXT,
    nombre_visible    TEXT,
    url_avatar        TEXT,
    url_linkedin      TEXT,                   -- perfil profesional
    url_curriculum    TEXT,                    -- enlace a CV (PDF en storage)
    activo            BOOLEAN DEFAULT TRUE,
    id_provincia      INTEGER REFERENCES provincias(id),
    ultimo_login      TIMESTAMPTZ,
    creado_en         TIMESTAMPTZ DEFAULT now(),
    actualizado_en    TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_usuarios_email ON usuarios (email);
CREATE INDEX idx_usuarios_provincia ON usuarios (id_provincia)
    WHERE id_provincia IS NOT NULL;

-- OAuth: Google, Apple — todos nullable excepto id e id_usuario
CREATE TABLE proveedores_autenticacion (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_usuario              UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    proveedor               TEXT,
    identificador_proveedor TEXT,
    email_proveedor         TEXT,
    datos_proveedor         JSONB,
    creado_en               TIMESTAMPTZ DEFAULT now(),
    actualizado_en          TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_proveedores_autenticacion_usuario
    ON proveedores_autenticacion (id_usuario);

-- Indice unico parcial: si ambos tienen valor, la combinacion es unica
CREATE UNIQUE INDEX idx_proveedores_autenticacion_unico
    ON proveedores_autenticacion (proveedor, identificador_proveedor)
    WHERE proveedor IS NOT NULL AND identificador_proveedor IS NOT NULL;

-- Sesiones JWT
CREATE TABLE tokens_refresco (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_usuario              UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    hash_token              TEXT UNIQUE,
    informacion_dispositivo TEXT,
    expira_en               TIMESTAMPTZ,
    revocado                BOOLEAN DEFAULT FALSE,
    creado_en               TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_tokens_refresco_usuario ON tokens_refresco (id_usuario);
CREATE INDEX idx_tokens_refresco_expira ON tokens_refresco (expira_en)
    WHERE revocado = FALSE;


-- ============================================================
-- DOMINIO 3: CONTENIDO
-- ============================================================

-- Ofertas de empleo (comunitarias, 0-N por usuario)
CREATE TABLE ofertas_empleo (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_autor            UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    titulo_puesto       TEXT,
    empresa             TEXT,
    ubicacion           TEXT,
    descripcion         TEXT,
    telefono_contacto   TEXT,
    email_contacto      TEXT,
    web_contacto        TEXT,
    activo              BOOLEAN DEFAULT TRUE,
    caduca_en           TIMESTAMPTZ,
    cantidad_upvotes    INTEGER DEFAULT 0,
    cantidad_downvotes  INTEGER DEFAULT 0,
    cantidad_reportes   INTEGER DEFAULT 0,
    estado_moderacion   estado_moderacion DEFAULT 'aprobado',
    creado_en           TIMESTAMPTZ DEFAULT now(),
    actualizado_en      TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_ofertas_autor ON ofertas_empleo (id_autor);
CREATE INDEX idx_ofertas_activas ON ofertas_empleo (activo, creado_en DESC)
    WHERE activo = TRUE;
CREATE INDEX idx_ofertas_caducidad ON ofertas_empleo (caduca_en)
    WHERE caduca_en IS NOT NULL AND activo = TRUE;
CREATE INDEX idx_ofertas_moderacion ON ofertas_empleo (estado_moderacion, creado_en)
    WHERE estado_moderacion != 'aprobado';

-- Consejos (comunitarios, escritos por usuarios)
CREATE TABLE consejos (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_autor            UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    titulo              TEXT,
    cuerpo              TEXT,
    web                 TEXT,                     -- enlace adicional opcional
    imagen_url          TEXT,
    activo              BOOLEAN DEFAULT TRUE,
    cantidad_upvotes    INTEGER DEFAULT 0,
    cantidad_downvotes  INTEGER DEFAULT 0,
    cantidad_reportes   INTEGER DEFAULT 0,
    estado_moderacion   estado_moderacion DEFAULT 'aprobado',
    creado_en           TIMESTAMPTZ DEFAULT now(),
    actualizado_en      TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_consejos_autor ON consejos (id_autor);
CREATE INDEX idx_consejos_activos ON consejos (activo, creado_en DESC)
    WHERE activo = TRUE;
CREATE INDEX idx_consejos_moderacion ON consejos (estado_moderacion, creado_en)
    WHERE estado_moderacion != 'aprobado';

-- Cursos (oficial por admin + comunitario por usuarios)
-- Campos adicionales para cursos comunitarios: descripcion, contenido,
-- duracion_horas, fecha_inicio, fecha_fin, curso_homologado, contacto
CREATE TABLE cursos (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_autor            UUID REFERENCES usuarios(id) ON DELETE SET NULL,
    titulo              TEXT,
    descripcion         TEXT,                     -- descripcion breve del curso
    contenido           TEXT,                     -- contenido detallado / temario
    web                 TEXT,                     -- enlace al curso
    imagen_url          TEXT,
    duracion_horas      INTEGER,                  -- numero de horas del curso
    fecha_inicio        DATE,
    fecha_fin           DATE,
    curso_homologado    BOOLEAN,                  -- true = oficial homologado
    telefono_contacto   TEXT,
    email_contacto      TEXT,
    origen              origen_contenido DEFAULT 'oficial',
    activo              BOOLEAN DEFAULT TRUE,
    cantidad_upvotes    INTEGER DEFAULT 0,
    cantidad_downvotes  INTEGER DEFAULT 0,
    cantidad_reportes   INTEGER DEFAULT 0,
    estado_moderacion   estado_moderacion DEFAULT 'aprobado',
    creado_en           TIMESTAMPTZ DEFAULT now(),
    actualizado_en      TIMESTAMPTZ DEFAULT now()
);

CREATE INDEX idx_cursos_autor ON cursos (id_autor) WHERE id_autor IS NOT NULL;
CREATE INDEX idx_cursos_origen ON cursos (origen, activo)
    WHERE activo = TRUE;
CREATE INDEX idx_cursos_activos ON cursos (activo, creado_en DESC)
    WHERE activo = TRUE;
CREATE INDEX idx_cursos_moderacion ON cursos (estado_moderacion, creado_en)
    WHERE estado_moderacion != 'aprobado';


-- ============================================================
-- DOMINIO 4: RELACIONES CONTENIDO <-> PROVINCIA
-- ============================================================

-- Ofertas <-> Provincias (N:M)
-- Una oferta puede estar en multiples provincias
CREATE TABLE ofertas_provincias (
    id_oferta       UUID NOT NULL REFERENCES ofertas_empleo(id) ON DELETE CASCADE,
    id_provincia    INTEGER NOT NULL REFERENCES provincias(id),
    PRIMARY KEY (id_oferta, id_provincia)
);

CREATE INDEX idx_ofertas_provincias_provincia ON ofertas_provincias (id_provincia);

-- Consejos <-> Provincias (N:M)
-- Sin filas = consejo nacional (aplica a toda Espana)
CREATE TABLE consejos_provincias (
    id_consejo      UUID NOT NULL REFERENCES consejos(id) ON DELETE CASCADE,
    id_provincia    INTEGER NOT NULL REFERENCES provincias(id),
    PRIMARY KEY (id_consejo, id_provincia)
);

CREATE INDEX idx_consejos_provincias_provincia ON consejos_provincias (id_provincia);

-- Cursos <-> Provincias (N:M)
CREATE TABLE cursos_provincias (
    id_curso        UUID NOT NULL REFERENCES cursos(id) ON DELETE CASCADE,
    id_provincia    INTEGER NOT NULL REFERENCES provincias(id),
    PRIMARY KEY (id_curso, id_provincia)
);

CREATE INDEX idx_cursos_provincias_provincia ON cursos_provincias (id_provincia);


-- ============================================================
-- DOMINIO 5: INTERACCIONES (polimorficas)
-- ============================================================

-- Votos (1 tabla polimorfica para ofertas + consejos + cursos)
--
-- tipo_voto INTEGER:
--    1  = upvote
--   -1  = downvote
--    0  = voto retirado (el usuario quito su voto)
--
-- tipo_contenido indica a que tabla apunta id_contenido:
--   'oferta'  -> ofertas_empleo
--   'consejo' -> consejos
--   'curso'   -> cursos
--
-- No hay FK en id_contenido porque PostgreSQL no soporta FK
-- condicionales a multiples tablas. La integridad se garantiza
-- desde el backend Rust.
--
-- El trigger actualizar_contadores_votos() actualiza automaticamente
-- los campos cantidad_upvotes y cantidad_downvotes en la tabla de
-- contenido correspondiente.
CREATE TABLE votos (
    id_usuario        UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    tipo_contenido    tipo_contenido NOT NULL,
    id_contenido      UUID NOT NULL,
    tipo_voto         INTEGER DEFAULT 0,      -- 1=upvote, -1=downvote, 0=sin voto
    creado_en         TIMESTAMPTZ DEFAULT now(),
    actualizado_en    TIMESTAMPTZ DEFAULT now(),
    PRIMARY KEY (id_usuario, tipo_contenido, id_contenido)
);

CREATE INDEX idx_votos_contenido ON votos (tipo_contenido, id_contenido);

-- Trigger: actualiza contadores en la tabla de contenido al votar
CREATE TRIGGER trg_votos_contadores
    AFTER INSERT OR UPDATE OR DELETE ON votos
    FOR EACH ROW
    EXECUTE FUNCTION actualizar_contadores_votos();

-- Reportes (1 tabla polimorfica para ofertas + consejos + cursos)
-- Misma logica polimorfica que votos
CREATE TABLE reportes (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_contenido    tipo_contenido,
    id_contenido      UUID,
    id_reportero      UUID NOT NULL REFERENCES usuarios(id) ON DELETE CASCADE,
    motivo            motivo_reporte,
    detalle_motivo    TEXT,
    estado            estado_reporte DEFAULT 'pendiente',
    id_procesador     UUID REFERENCES usuarios(id),
    procesado_en      TIMESTAMPTZ,
    creado_en         TIMESTAMPTZ DEFAULT now(),
    actualizado_en    TIMESTAMPTZ DEFAULT now(),
    UNIQUE (tipo_contenido, id_contenido, id_reportero)
);

CREATE INDEX idx_reportes_pendientes ON reportes (estado, creado_en)
    WHERE estado = 'pendiente';
CREATE INDEX idx_reportes_contenido ON reportes (tipo_contenido, id_contenido);


-- ============================================================
-- DOMINIO 6: SISTEMA
-- ============================================================

-- Configuracion global de la aplicacion (key-value)
-- Permite cambiar comportamiento de la app sin redesplegar el backend.
-- Ejemplos:
--   clave: 'modo_mantenimiento'      -> valor: 'false'
--   clave: 'version_minima_app'      -> valor: '2.0.0'
--   clave: 'max_ofertas_por_usuario' -> valor: '50'
--   clave: 'umbral_reportes_ocultar' -> valor: '5'
CREATE TABLE configuracion_aplicacion (
    clave           TEXT PRIMARY KEY,
    valor           TEXT,
    descripcion     TEXT,
    actualizado_en  TIMESTAMPTZ DEFAULT now()
);

-- Prestaciones SEPE nacionales (RAI, SED, etc.)
-- Datos de requirements.json — son nacionales, NO per-provincia
CREATE TABLE prestaciones (
    id              SERIAL PRIMARY KEY,
    titulo          TEXT,
    descripcion     TEXT,
    requisitos      TEXT[],
    url             TEXT,
    activo          BOOLEAN DEFAULT TRUE,
    creado_en       TIMESTAMPTZ DEFAULT now(),
    actualizado_en  TIMESTAMPTZ DEFAULT now()
);


-- ============================================================
-- TRIGGERS actualizado_en
-- ============================================================

CREATE TRIGGER trg_comunidades_actualizado BEFORE UPDATE ON comunidades_autonomas
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_provincias_actualizado BEFORE UPDATE ON provincias
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_oficinas_actualizado BEFORE UPDATE ON oficinas_sepe
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_usuarios_actualizado BEFORE UPDATE ON usuarios
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_ofertas_actualizado BEFORE UPDATE ON ofertas_empleo
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_consejos_actualizado BEFORE UPDATE ON consejos
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_cursos_actualizado BEFORE UPDATE ON cursos
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_votos_actualizado BEFORE UPDATE ON votos
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_reportes_actualizado BEFORE UPDATE ON reportes
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_configuracion_actualizado BEFORE UPDATE ON configuracion_aplicacion
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();
CREATE TRIGGER trg_prestaciones_actualizado BEFORE UPDATE ON prestaciones
    FOR EACH ROW EXECUTE FUNCTION set_actualizado_en();

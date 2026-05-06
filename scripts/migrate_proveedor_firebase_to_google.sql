-- ============================================================
-- Migracion idempotente: proveedor 'firebase' -> 'google.com'
-- ============================================================
--
-- Contexto:
--   Antes del refactor de auth, todas las identidades Firebase (que solo
--   soportaban Google) se guardaban con `proveedor = 'firebase'` en
--   `proveedores_autenticacion`.
--
--   El nuevo backend acepta cualquier `sign_in_provider` Firebase
--   ('google.com', 'password', 'anonymous') y guarda el literal real,
--   por lo que las filas legacy de Google deben renombrarse para que el
--   lookup `(proveedor, identificador_proveedor)` siga encontrandolas.
--
-- Cuando ejecutar:
--   ANTES de desplegar el backend nuevo. Si se despliega primero el
--   binario, los usuarios Google legacy seran tratados como nuevos en su
--   siguiente login (creara una segunda fila con proveedor='google.com'
--   y autoenlazara por email; no se pierden datos pero queda ruido).
--
-- Caracteristicas:
--   - Idempotente: el WHERE filtra solo las filas que aun tienen el
--     valor antiguo, asi que correrlo varias veces es seguro.
--   - Transaccional: o se aplica todo o no se aplica nada.
--   - Toca el campo `actualizado_en` para que quede trazable en logs.
--
-- Uso:
--   psql "$DATABASE_URL" -f scripts/migrate_proveedor_firebase_to_google.sql
--
-- Rollback (si fuera necesario):
--   UPDATE proveedores_autenticacion
--      SET proveedor = 'firebase'
--    WHERE proveedor = 'google.com';
-- ============================================================

BEGIN;

UPDATE proveedores_autenticacion
   SET proveedor      = 'google.com',
       actualizado_en = now()
 WHERE proveedor = 'firebase';

COMMIT;

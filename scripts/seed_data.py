#!/usr/bin/env python3
"""
seed_data.py — Genera seed.sql para la BBDD InemSellar (PostgreSQL)

Lee datos de:
  - inemsellar_app/assets/data/sel.json   → comunidades_autonomas
  - inemsellar_app/assets/data/provincias.csv → provincias
  - inemsellar_app/assets/data/tel.json   → oficinas_sepe (teléfonos)
  - inemsellar_app/assets/data/cur.json   → cursos + oficinas_sepe (url_cursos)
  - inemsellar_app/assets/data/con.json   → consejos
  - DB INEM SELLA - Hoja 1.csv            → consejos adicionales
  - ayudas.csv                             → prestaciones
  - assets/data/requirements.json          → prestaciones (requisitos)

Output: scripts/seed.sql
"""

import json
import csv
import os
import re
import uuid
from pathlib import Path

# ============================================================
# PATHS
# ============================================================
APP_DIR = Path("/home/admin/companies/apptolast/InnemSellarApliArte/inemsellar_app")
BACKEND_DIR = Path("/home/admin/companies/apptolast/InnemBackendDespliegue")
OUTPUT_SQL = BACKEND_DIR / "scripts" / "seed.sql"

# Source files
SEL_JSON = APP_DIR / "assets" / "data" / "sel.json"
CUR_JSON = APP_DIR / "assets" / "data" / "cur.json"
CON_JSON = APP_DIR / "assets" / "data" / "con.json"
TEL_JSON = APP_DIR / "assets" / "data" / "tel.json"
PROV_CSV = APP_DIR / "assets" / "data" / "provincias.csv"
REQ_JSON = APP_DIR / "assets" / "data" / "requirements.json"
AYUDAS_CSV = APP_DIR / "ayudas.csv"
DB_CSV = APP_DIR / "DB INEM SELLA - Hoja 1.csv"


def load_json(path):
    with open(path, "r", encoding="utf-8") as f:
        data = json.load(f)
    return [item for item in data if item.get("id") != "guia_campos"]


def load_csv(path, encoding="utf-8-sig"):
    with open(path, "r", encoding=encoding) as f:
        return list(csv.DictReader(f))


def sql_escape(value):
    """Escape a string for SQL single-quoted literal."""
    if value is None:
        return "NULL"
    s = str(value).strip()
    if not s:
        return "NULL"
    s = s.replace("'", "''")
    # Remove control characters except newlines
    s = re.sub(r'[\x00-\x09\x0b\x0c\x0e-\x1f]', '', s)
    return f"'{s}'"


def sql_text_array(items):
    """Convert a Python list to a PostgreSQL TEXT[] literal."""
    if not items:
        return "NULL"
    escaped = []
    for item in items:
        s = str(item).replace('"', '\\"').replace("'", "''")
        escaped.append(f'"{s}"')
    return "'{" + ",".join(escaped) + "}'"


def normalize_title(title):
    """Normalize title for dedup comparison."""
    if not title:
        return ""
    t = title.lower().strip()
    # Remove accents (simplified)
    replacements = {
        'á': 'a', 'é': 'e', 'í': 'i', 'ó': 'o', 'ú': 'u',
        'ñ': 'n', 'ü': 'u'
    }
    for old, new in replacements.items():
        t = t.replace(old, new)
    # Remove punctuation
    t = re.sub(r'[¿?¡!.,;:\-\s]+', ' ', t).strip()
    return t


def clean_html(text):
    """Remove HTML tags from text."""
    if not text:
        return None
    # Replace common HTML entities
    text = text.replace("&nbsp;", " ")
    text = text.replace("&amp;", "&")
    text = text.replace("&lt;", "<")
    text = text.replace("&gt;", ">")
    # Remove HTML tags
    text = re.sub(r'<[^>]+>', '', text)
    # Normalize whitespace
    text = re.sub(r'\s+', ' ', text).strip()
    return text if text else None


# ============================================================
# DATA: Comunidades Autonomas
# ============================================================
# Mapping from sel.json id → (community_id, nombre, nombre_servicio_empleo)
COMUNIDADES = [
    (1, "sel_andalucia", "Andalucía", "SAE"),
    (2, "sel_aragon", "Aragón", "INAEM"),
    (3, "sel_asturias", "Asturias", "Trabajastur"),
    (4, "sel_baleares", "Illes Balears", "SOIB"),
    (5, "sel_canarias", "Canarias", "SCE"),
    (6, "sel_cantabria", "Cantabria", "EMCAN"),
    (7, "sel_cyl", "Castilla y León", "ECYL"),
    (8, "sel_clm", "Castilla-La Mancha", "SEPECAM"),
    (9, "sel_catalunya", "Cataluña", "SOC"),
    (10, "sel_valencia", "Comunitat Valenciana", "Labora"),
    (11, "sel_extremadura", "Extremadura", "SEXPE"),
    (12, "sel_galicia", "Galicia", "Emprego Xunta"),
    (13, "sel_madrid", "Madrid", "Comunidad de Madrid"),
    (14, "sel_murcia", "Murcia", "SEF"),
    (15, "sel_navarra", "Navarra", "SNE Navarra"),
    (16, "sel_euskadi", "País Vasco", "Lanbide"),
    (17, "sel_rioja", "La Rioja", "Empleo La Rioja"),
    (18, "sel_ceuta", "Ceuta", "SEPE Ceuta"),
    (19, "sel_melilla", "Melilla", "SEPE Melilla"),
]

# Province ID → Community ID mapping (derived from sel.json tags)
# Province IDs match the CSV (2-52) + 53 for Alicante
PROVINCIA_COMUNIDAD = {
    # Andalucía (1)
    4: 1, 13: 1, 17: 1, 21: 1, 24: 1, 26: 1, 32: 1, 42: 1,
    # Aragón (2)
    25: 2, 45: 2, 50: 2,
    # Asturias (3)
    5: 3,
    # Illes Balears (4)
    8: 4,
    # Canarias (5)
    37: 5, 40: 5,
    # Cantabria (6)
    14: 6,
    # Castilla y León (7)
    6: 7, 11: 7, 27: 7, 36: 7, 39: 7, 41: 7, 43: 7, 48: 7, 49: 7,
    # Castilla-La Mancha (8)
    3: 8, 16: 8, 19: 8, 22: 8, 46: 8,
    # Cataluña (9)
    9: 9, 20: 9, 28: 9, 44: 9,
    # Comunitat Valenciana (10)
    15: 10, 47: 10, 53: 10,  # 53 = Alicante (nueva)
    # Extremadura (11)
    7: 11, 12: 11,
    # Galicia (12)
    18: 12, 30: 12, 35: 12, 38: 12,
    # Madrid (13)
    31: 13,
    # Murcia (14)
    33: 14,
    # Navarra (15)
    34: 15,
    # País Vasco (16)
    2: 16, 10: 16, 23: 16,
    # La Rioja (17)
    29: 17,
    # Ceuta (18)
    51: 18,
    # Melilla (19)
    52: 19,
}


def generate_sql():
    lines = []
    lines.append("-- ============================================================")
    lines.append("-- InemSellar — Seed Data")
    lines.append("-- Generado automaticamente por seed_data.py")
    lines.append("-- ============================================================")
    lines.append("")
    lines.append("BEGIN;")
    lines.append("")

    # ----------------------------------------------------------
    # Pre-step: ALTER consejos.id_autor to nullable
    # ----------------------------------------------------------
    lines.append("-- Pre-paso: hacer consejos.id_autor nullable (como cursos.id_autor)")
    lines.append("ALTER TABLE consejos ALTER COLUMN id_autor DROP NOT NULL;")
    lines.append("ALTER TABLE consejos DROP CONSTRAINT IF EXISTS consejos_id_autor_fkey;")
    lines.append("ALTER TABLE consejos ADD CONSTRAINT consejos_id_autor_fkey")
    lines.append("  FOREIGN KEY (id_autor) REFERENCES usuarios(id) ON DELETE SET NULL;")
    lines.append("")

    # ----------------------------------------------------------
    # 1. Comunidades Autonomas
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 1. COMUNIDADES AUTONOMAS (19 registros)")
    lines.append("-- ============================================================")

    sel_data = load_json(SEL_JSON)
    sel_by_id = {item["id"]: item for item in sel_data}

    for com_id, sel_id, nombre, servicio in COMUNIDADES:
        sel_item = sel_by_id.get(sel_id, {})
        url_sellado = sel_item.get("url", "")

        lines.append(
            f"INSERT INTO comunidades_autonomas (id, nombre, nombre_servicio_empleo, url_sellado) "
            f"VALUES ({com_id}, {sql_escape(nombre)}, {sql_escape(servicio)}, {sql_escape(url_sellado)});"
        )

    # Reset sequence
    lines.append("SELECT setval('comunidades_autonomas_id_seq', 19);")
    lines.append("")

    # ----------------------------------------------------------
    # 2. Provincias
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 2. PROVINCIAS (52 registros)")
    lines.append("-- ============================================================")

    prov_data = load_csv(PROV_CSV)
    for row in prov_data:
        prov_id = int(row["id"])
        nombre = row["provincia"].strip()

        # Skip "De Pago" (ID 1) - not a real province
        if prov_id == 1:
            continue

        com_id = PROVINCIA_COMUNIDAD.get(prov_id)
        if com_id is None:
            print(f"WARNING: Provincia ID {prov_id} ({nombre}) sin comunidad mapeada, saltando")
            continue

        lines.append(
            f"INSERT INTO provincias (id, nombre, id_comunidad) "
            f"VALUES ({prov_id}, {sql_escape(nombre)}, {com_id});"
        )

    # Add Alicante (ID 53, Com. Valenciana = 10)
    lines.append(
        f"INSERT INTO provincias (id, nombre, id_comunidad) "
        f"VALUES (53, 'Alicante', 10);"
    )
    lines.append("")

    # ----------------------------------------------------------
    # 3. Oficinas SEPE
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 3. OFICINAS SEPE (52 registros)")
    lines.append("-- ============================================================")

    # Load phone data from tel.json
    tel_data = load_json(TEL_JSON)
    # Map tag (pN) → phone number
    tel_by_prov = {}
    for item in tel_data:
        tags = item.get("tags", [])
        phone = item.get("tel", "")
        for tag in tags:
            if tag.startswith("p"):
                try:
                    prov_id = int(tag[1:])
                    tel_by_prov[prov_id] = phone
                except ValueError:
                    pass

    # Load course URLs from cur.json
    cur_data = load_json(CUR_JSON)
    # Map province_id → course URL (via community tags)
    curso_url_by_prov = {}
    for item in cur_data:
        url = item.get("url", "")
        if not url:
            continue
        tags = item.get("tags", [])
        for tag in tags:
            if tag.startswith("p"):
                try:
                    prov_id = int(tag[1:])
                    curso_url_by_prov[prov_id] = url
                except ValueError:
                    pass

    # Also map communities without specific province tags to all their provinces
    # For cur.json entries with empty tags, try to match by community name
    for item in cur_data:
        url = item.get("url", "")
        if not url:
            continue
        tags = item.get("tags", [])
        if tags:
            continue
        # Empty tags — try to assign to provinces by matching title
        title = item.get("title", "").lower()
        if "baleares" in title:
            curso_url_by_prov.setdefault(8, url)
        elif "valencia" in title:
            curso_url_by_prov.setdefault(15, url)
            curso_url_by_prov.setdefault(47, url)
            curso_url_by_prov.setdefault(53, url)  # Alicante

    # Generate oficinas for all provinces (2-52 + 53)
    all_prov_ids = sorted(
        [pid for pid in PROVINCIA_COMUNIDAD.keys()]
    )

    for prov_id in all_prov_ids:
        phone = tel_by_prov.get(prov_id, None)
        url_cursos = curso_url_by_prov.get(prov_id, None)

        lines.append(
            f"INSERT INTO oficinas_sepe (id_provincia, telefono, url_cursos) "
            f"VALUES ({prov_id}, {sql_escape(phone)}, {sql_escape(url_cursos)});"
        )

    lines.append("")

    # ----------------------------------------------------------
    # 4. Consejos (deduplicados de con.json + DB INEM SELLA CSV)
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 4. CONSEJOS (deduplicados)")
    lines.append("-- ============================================================")

    # Load con.json items
    con_data = load_json(CON_JSON)
    consejos = {}  # normalized_title → {titulo, cuerpo, web, imagen_url}

    for item in con_data:
        titulo = item.get("title", "")
        if not titulo:
            continue
        key = normalize_title(titulo)
        contenido = clean_html(item.get("contenido", ""))
        web = item.get("url", "")
        imagen = item.get("imagenUrl", "")

        consejos[key] = {
            "titulo": titulo,
            "cuerpo": contenido,
            "web": web if web else None,
            "imagen_url": imagen if imagen else None,
        }

    # Load DB INEM SELLA CSV items with categoria="con"
    db_csv_data = load_csv(DB_CSV)
    for row in db_csv_data:
        cat = row.get("categoria", "").strip()
        if cat != "con":
            continue
        titulo = row.get("titulo", "").strip()
        if not titulo:
            continue
        key = normalize_title(titulo)
        # Only add if not already from con.json (con.json has richer data)
        if key not in consejos:
            descripcion = clean_html(row.get("descripcion", ""))
            web = row.get("enlaceOTexto", "").strip()
            imagen = row.get("imagen", "").strip()

            consejos[key] = {
                "titulo": titulo,
                "cuerpo": descripcion,
                "web": web if web else None,
                "imagen_url": imagen if imagen else None,
            }

    consejo_count = 0
    for key, data in sorted(consejos.items()):
        consejo_uuid = str(uuid.uuid4())
        lines.append(
            f"INSERT INTO consejos (id, id_autor, titulo, cuerpo, web, imagen_url, activo, estado_moderacion) "
            f"VALUES ('{consejo_uuid}', NULL, {sql_escape(data['titulo'])}, "
            f"{sql_escape(data['cuerpo'])}, {sql_escape(data['web'])}, "
            f"{sql_escape(data['imagen_url'])}, TRUE, 'aprobado');"
        )
        consejo_count += 1

    lines.append(f"-- Total consejos: {consejo_count}")
    lines.append("")

    # ----------------------------------------------------------
    # 5. Cursos + cursos_provincias
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 5. CURSOS (18 registros) + CURSOS_PROVINCIAS")
    lines.append("-- ============================================================")

    curso_uuids = {}  # cur_id → uuid
    for item in cur_data:
        cur_id = item.get("id", "")
        titulo = item.get("title", "")
        web = item.get("url", "")
        if not titulo:
            continue

        curso_uuid = str(uuid.uuid4())
        curso_uuids[cur_id] = curso_uuid

        lines.append(
            f"INSERT INTO cursos (id, id_autor, titulo, web, origen, activo, estado_moderacion) "
            f"VALUES ('{curso_uuid}', NULL, {sql_escape(titulo)}, "
            f"{sql_escape(web)}, 'oficial', TRUE, 'aprobado');"
        )

    lines.append("")
    lines.append("-- Cursos ↔ Provincias (N:M)")

    for item in cur_data:
        cur_id = item.get("id", "")
        curso_uuid = curso_uuids.get(cur_id)
        if not curso_uuid:
            continue
        tags = item.get("tags", [])
        for tag in tags:
            if tag.startswith("p"):
                try:
                    prov_id = int(tag[1:])
                    # Only insert if province exists
                    if prov_id in PROVINCIA_COMUNIDAD:
                        lines.append(
                            f"INSERT INTO cursos_provincias (id_curso, id_provincia) "
                            f"VALUES ('{curso_uuid}', {prov_id});"
                        )
                except ValueError:
                    pass

    lines.append("")

    # ----------------------------------------------------------
    # 6. Prestaciones
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 6. PRESTACIONES")
    lines.append("-- ============================================================")

    # Load requirements.json for RAI and SED requisitos
    req_data = load_json(REQ_JSON)
    req_by_title = {}
    for item in req_data:
        title = item.get("title", "")
        req_by_title[normalize_title(title)] = item

    # Load ayudas.csv
    ayudas_data = load_csv(AYUDAS_CSV)

    for row in ayudas_data:
        titulo = row.get("Titulo", "").strip()
        descripcion = row.get("Descripcion", "").strip()
        enlace = row.get("Enlace", "").strip()
        extras = row.get("Extras", "").strip()

        # Check if this matches a requirements.json entry (for RAI/SED)
        norm_title = normalize_title(titulo)
        req_match = None
        for req_key, req_item in req_by_title.items():
            # Fuzzy match: check if key words overlap
            if "rai" in norm_title and "rai" in req_key:
                req_match = req_item
                break
            if "sed" in norm_title and "sed" in req_key:
                req_match = req_item
                break
            if "extraordinario" in norm_title and "extraordinario" in req_key:
                req_match = req_item
                break

        # Build requisitos array
        requisitos = None
        if req_match and req_match.get("requirements"):
            requisitos = req_match["requirements"]
        elif extras:
            # Parse extras like "age:over_52, status:exhausted, income:low"
            requisitos = [tag.strip() for tag in extras.split(",") if tag.strip()]

        lines.append(
            f"INSERT INTO prestaciones (titulo, descripcion, requisitos, url, activo) "
            f"VALUES ({sql_escape(titulo)}, {sql_escape(descripcion)}, "
            f"{sql_text_array(requisitos)}, {sql_escape(enlace)}, TRUE);"
        )

    lines.append("")

    # ----------------------------------------------------------
    # 7. Configuracion de la aplicacion
    # ----------------------------------------------------------
    lines.append("-- ============================================================")
    lines.append("-- 7. CONFIGURACION APLICACION")
    lines.append("-- ============================================================")

    config_items = [
        ("modo_mantenimiento", "false", "Activa/desactiva modo mantenimiento"),
        ("version_minima_app", "2.0.0", "Version minima requerida de la app"),
        ("max_ofertas_por_usuario", "50", "Limite de ofertas activas por usuario"),
        ("umbral_reportes_ocultar", "5", "Reportes necesarios para ocultar contenido"),
        ("telefono_sepe_pago", "901010210", "Numero de telefono de pago del SEPE"),
    ]

    # Add "ini" items from DB INEM SELLA CSV
    for row in db_csv_data:
        cat = row.get("categoria", "").strip()
        if cat != "ini":
            continue
        titulo = row.get("titulo", "").strip()
        enlace = row.get("enlaceOTexto", "").strip()
        if not titulo or not enlace:
            continue
        # Skip version row
        if "version" in titulo.lower():
            continue
        # Convert title to key
        key = "url_" + re.sub(r'[^a-z0-9]+', '_', titulo.lower().strip()).strip('_')
        config_items.append((key, enlace, titulo))

    for clave, valor, descripcion in config_items:
        lines.append(
            f"INSERT INTO configuracion_aplicacion (clave, valor, descripcion) "
            f"VALUES ({sql_escape(clave)}, {sql_escape(valor)}, {sql_escape(descripcion)});"
        )

    lines.append("")
    lines.append("COMMIT;")
    lines.append("")
    lines.append("-- FIN del seed")
    lines.append("")

    return "\n".join(lines)


if __name__ == "__main__":
    sql = generate_sql()
    with open(OUTPUT_SQL, "w", encoding="utf-8") as f:
        f.write(sql)
    print(f"Seed SQL generado en: {OUTPUT_SQL}")
    print(f"Tamaño: {len(sql)} bytes")

    # Count statements
    inserts = sql.count("INSERT INTO")
    print(f"Total INSERTs: {inserts}")

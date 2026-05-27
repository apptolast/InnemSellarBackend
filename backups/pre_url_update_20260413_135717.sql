--
-- PostgreSQL database dump
--

\restrict IqnPGtSBgodJkyoQehbSGECzFaXiW5JNTLAQ0Gehb7cZa3AwFt8HUPgZ6TvqbuR

-- Dumped from database version 16.10
-- Dumped by pg_dump version 16.10

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Data for Name: comunidades_autonomas; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (1, 'Andalucía', 'SAE', NULL, 'https://ws054.juntadeandalucia.es/autenticacion/login?service=https%3A%2F%2Fws054.juntadeandalucia.es%2Fapdweb%2Fweb%2Fguest%2Fhome', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (2, 'Aragón', 'INAEM', NULL, 'http://www.sistemanacionalempleo.es/inicio/sne/CartaServiciosCiudadanoWEB/jspFR/principalFR.jsp?CA=02', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (3, 'Asturias', 'Trabajastur', NULL, 'http://www.asturias.es/portal/site/trabajastur', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (4, 'Illes Balears', 'SOIB', NULL, 'http://www.caib.es/govern/sac/fitxa.do?estua=1464&lang=es&codi=597679&coduo=1464', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (5, 'Canarias', 'SCE', NULL, 'https://www3.gobiernodecanarias.org/empleo/portal/web/sce/sede_electronica/desempleados/renovacion_demanda', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (6, 'Cantabria', 'EMCAN', NULL, 'https://www.empleacantabria.es/demanda-de-empleo-cantabria', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (7, 'Castilla y León', 'ECYL', NULL, 'https://empleocastillayleon.jcyl.es/oficinavirtual/formularioCitaPreviaPF.do?srvc=inicio', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (8, 'Castilla-La Mancha', 'SEPECAM', NULL, 'https://e-empleo.jccm.es/demandantenew/jsp/servdemandante.jsp', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (9, 'Cataluña', 'SOC', NULL, 'https://www.oficinadetreball.gencat.cat/socfuncions/Renovacio.do?idiomaNavegacio=ca&secure=S', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (10, 'Comunitat Valenciana', 'Labora', NULL, 'http://www.ocupacio.gva.es:7017/portal/web/home/renovaciodarde', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (11, 'Extremadura', 'SEXPE', NULL, 'http://www.extremaduratrabaja.es/ciudadanos/tramites-y-prestaciones/renovacion-de-demanda', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (12, 'Galicia', 'Emprego Xunta', NULL, 'https://emprego.xunta.gal/portal/es/11-demandantes/demandantes-sp/389-renovacion-demanda.html', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (13, 'Madrid', 'Comunidad de Madrid', NULL, 'https://www.comunidad.madrid/servicios/empleo/gestion-telematica-demanda-empleo', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (14, 'Murcia', 'SEF', NULL, 'http://www.sefcarm.es/web/pagina?IDCONTENIDO=5173&IDTIPO=100&RASTRO=c$m29962,30020', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (15, 'Navarra', 'SNE Navarra', NULL, 'http://www.navarra.es/home_es/Temas/Empleo+y+Economia/Empleo/PreviaOfiElect.htm', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (16, 'País Vasco', 'Lanbide', NULL, 'http://www.lanbide.net', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (17, 'La Rioja', 'Empleo La Rioja', NULL, 'http://www.larioja.org/npRioja/default/defaultpage.jsp?idtab=432229&IdDoc=522936', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (18, 'Ceuta', 'SEPE Ceuta', NULL, 'https://sede.sepe.gob.es/portalSedeEstaticos/Redirect.do?page=cb00', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.comunidades_autonomas (id, nombre, nombre_servicio_empleo, web_servicio_empleo, url_sellado, creado_en, actualizado_en) VALUES (19, 'Melilla', 'SEPE Melilla', NULL, 'https://sede.sepe.gob.es/portalSedeEstaticos/Redirect.do?page=cb00', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');


--
-- Data for Name: cursos; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('7f324ff5-0869-45d9-9f65-52092e57a29c', NULL, 'Cursos INEM en Baleares', NULL, NULL, 'http://www.caib.es/sacmicrofront/contenido.do?mkey=M10011509503311893878&cont=15950&&lang=es', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', NULL, 'Cursos INEM en Castilla-La Mancha', NULL, NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('7d3518de-b031-442a-baae-9d44df8ed491', NULL, 'Cursos INEM en Cataluña', NULL, NULL, 'https://cursosinemweb.es/cursos-inem-cataluna/', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('58f16b73-5188-475d-a3d4-2ca7f0d9acb8', NULL, 'Cursos INEM en Ceuta', NULL, NULL, 'http://www.citapreviainem.es/cursos-inem-ceuta/', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('8a9ab98b-b29f-4f42-8f92-5857208ee309', NULL, 'Cursos INEM en Extremadura', NULL, NULL, 'http://extremaduratrabaja.gobex.es/ciudadanos/formacion/instituto-extreme%C3%B1o-de-las-cualificaciones-y-acreditaciones', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('5866fc8f-dcf7-43f2-8c38-42bb1699fde9', NULL, 'Cursos INEM en Galicia', NULL, NULL, 'http://traballo.xunta.es/cursos-dirixidos-a-traballadores-desempregados', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('12ddfd5c-ad43-45b9-92aa-0a42c851ed25', NULL, 'Cursos INEM en La Rioja', NULL, NULL, 'http://www.larioja.org/npRioja/default/defaultpage.jsp?idtab=423414', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('9e22b352-48f6-4b92-9ac7-e4484810cecc', NULL, 'Cursos INEM en Madrid', NULL, NULL, 'http://www.madrid.org/cs/Satellite?cid=1142337223387&language=es&pagename=Empleo%2FPage%2FEMPL_pintarContenidoFinal', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('7fa38c1d-0cbd-4ae9-bd68-6f74d3579731', NULL, 'Cursos INEM en Melilla', NULL, NULL, 'http://www.melillaorienta.es/?page=listadocursos', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('43792a28-318c-49e1-b2d5-f1bfedbe1418', NULL, 'Cursos INEM en Murcia', NULL, NULL, 'http://www.sefcarm.es/web/pagina?IDCONTENIDO=30040&IDTIPO=100&RASTRO=c$m29962', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('67e8e85f-5691-463a-ab12-ca1aa9bfbe89', NULL, 'Cursos INEM en Navarra', NULL, NULL, 'http://www.navarra.es/home_es/Temas/Empleo+y+Economia/Empleo/Formacion/Personas+desempleadas/Default.htm', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('3964ec07-4a9a-4c46-9d23-7e72eea697a0', NULL, 'Cursos INEM en Valencia', NULL, NULL, 'http://www.ocupacio.gva.es:7017/portal/web/home/OfertaCursos2/BuscarCursos', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('805bd218-bff6-4c98-ad51-c80cf8bda132', NULL, 'Cursos INEM en el País Vasco', NULL, NULL, 'http://www.lanbide.net/plsql/fr_menu?idioma=C', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('b54a0f19-0732-4a20-9c32-8f8bf48a52f5', NULL, 'para mas  cursos en españa os recomiendo esta web', NULL, NULL, 'https://cursosinemweb.es/cursos-inem/', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('c99abdad-dbe1-4329-a3bd-31e7e20057e2', 'd84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'Curso Test', 'Descripción test', 'Contenido y temario', NULL, NULL, 2000, NULL, NULL, false, NULL, NULL, 'comunidad', true, 1, 0, 0, 'aprobado', '2026-04-11 20:22:25.188919+00', '2026-04-12 00:33:36.903516+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('a13d6d64-5ddc-4005-89b2-cbf738cebe43', 'ef8a04d0-27f6-4121-96a2-ca0e0fad2ffe', 'Curso de Introducción al Test', 'TestOrigen', 'Test', 'https://midominio.com', '', 400, '2026-05-02', '2026-05-20', false, '+34600123456', 'b@b.com', 'comunidad', true, 0, 1, 0, 'aprobado', '2026-04-11 17:03:31.886684+00', '2026-04-11 23:14:48.276653+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('2073120f-24c0-4d3f-9f0c-f47773c663a2', NULL, 'Cursos INEM en Canarias', NULL, NULL, 'http://www3.gobiernodecanarias.org/empleo/portal/web/sce/servicios/cursos?inicio=false&q=&isla=&municipioCache=&modalidad=&sinMunicipio=true&tipoCurso=desempleados', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 1, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-12 03:11:24.579092+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', NULL, 'Cursos INEM en Castilla y León', NULL, NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 0, 1, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-12 03:16:55.10769+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('c9d9325b-889c-4e21-a957-62a040cb1515', NULL, 'Cursos INEM en Aragón', NULL, NULL, 'http://plan.aragon.es/MapaRec.nsf/General', NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 1, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-11 23:50:43.967963+00');
INSERT INTO public.cursos (id, id_autor, titulo, descripcion, contenido, web, imagen_url, duracion_horas, fecha_inicio, fecha_fin, curso_homologado, telefono_contacto, email_contacto, origen, activo, cantidad_upvotes, cantidad_downvotes, cantidad_reportes, estado_moderacion, creado_en, actualizado_en) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', NULL, 'Cursos INEM en Andalucía', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, 'oficial', true, 1, 0, 0, 'aprobado', '2026-04-08 12:31:51.943828+00', '2026-04-12 17:34:42.435241+00');


--
-- Data for Name: cursos_provincias; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 4);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 13);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 17);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 21);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 24);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 26);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 32);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 42);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('c9d9325b-889c-4e21-a957-62a040cb1515', 25);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('c9d9325b-889c-4e21-a957-62a040cb1515', 45);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('c9d9325b-889c-4e21-a957-62a040cb1515', 50);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('2073120f-24c0-4d3f-9f0c-f47773c663a2', 37);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('2073120f-24c0-4d3f-9f0c-f47773c663a2', 40);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 6);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 11);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 27);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 36);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 39);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 41);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 43);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 48);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5a35887f-11e6-4f31-a068-57c63d36c833', 49);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', 3);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', 16);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', 19);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', 22);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('4fa2ccca-e09a-43c1-bde5-cb91b4784725', 46);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('7d3518de-b031-442a-baae-9d44df8ed491', 9);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('7d3518de-b031-442a-baae-9d44df8ed491', 20);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('7d3518de-b031-442a-baae-9d44df8ed491', 28);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('7d3518de-b031-442a-baae-9d44df8ed491', 44);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('58f16b73-5188-475d-a3d4-2ca7f0d9acb8', 51);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('8a9ab98b-b29f-4f42-8f92-5857208ee309', 7);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('8a9ab98b-b29f-4f42-8f92-5857208ee309', 12);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5866fc8f-dcf7-43f2-8c38-42bb1699fde9', 18);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5866fc8f-dcf7-43f2-8c38-42bb1699fde9', 30);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5866fc8f-dcf7-43f2-8c38-42bb1699fde9', 35);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('5866fc8f-dcf7-43f2-8c38-42bb1699fde9', 38);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('12ddfd5c-ad43-45b9-92aa-0a42c851ed25', 29);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('9e22b352-48f6-4b92-9ac7-e4484810cecc', 31);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('7fa38c1d-0cbd-4ae9-bd68-6f74d3579731', 52);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('43792a28-318c-49e1-b2d5-f1bfedbe1418', 33);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('67e8e85f-5691-463a-ab12-ca1aa9bfbe89', 34);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('805bd218-bff6-4c98-ad51-c80cf8bda132', 2);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('805bd218-bff6-4c98-ad51-c80cf8bda132', 10);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('805bd218-bff6-4c98-ad51-c80cf8bda132', 23);
INSERT INTO public.cursos_provincias (id_curso, id_provincia) VALUES ('a13d6d64-5ddc-4005-89b2-cbf738cebe43', 2);


--
-- Data for Name: oficinas_sepe; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (1, 2, '945750579', NULL, 'http://www.lanbide.net/plsql/fr_menu?idioma=C', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (2, 3, '967750579', NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (3, 4, '950750579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (4, 5, '984751579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (5, 6, '920750779', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (6, 7, '924990579', NULL, 'http://extremaduratrabaja.gobex.es/ciudadanos/formacion/instituto-extreme%C3%B1o-de-las-cualificaciones-y-acreditaciones', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (7, 8, '971980779', NULL, 'http://www.caib.es/sacmicrofront/contenido.do?mkey=M10011509503311893878&cont=15950&&lang=es', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (8, 9, '936190579', NULL, 'https://cursosinemweb.es/cursos-inem-cataluna/', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (9, 10, '944506579', NULL, 'http://www.lanbide.net/plsql/fr_menu?idioma=C', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (10, 11, '947750879', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (11, 12, '927750579', NULL, 'http://extremaduratrabaja.gobex.es/ciudadanos/formacion/instituto-extreme%C3%B1o-de-las-cualificaciones-y-acreditaciones', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (12, 13, '956992579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (13, 14, '942990579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (14, 15, '964750579', NULL, 'http://www.ocupacio.gva.es:7017/portal/web/home/OfertaCursos2/BuscarCursos', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (15, 16, '926990579', NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (16, 17, '957990579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (17, 18, '981995579', NULL, 'http://traballo.xunta.es/cursos-dirixidos-a-traballadores-desempregados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (18, 19, '969750579', NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (19, 20, '972068679', NULL, 'https://cursosinemweb.es/cursos-inem-cataluna/', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (20, 21, '958900879', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (21, 22, '949750979', NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (22, 23, '943980579', NULL, 'http://www.lanbide.net/plsql/fr_menu?idioma=C', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (23, 24, '959750579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (24, 25, '974750579', NULL, 'http://plan.aragon.es/MapaRec.nsf/General', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (25, 26, '953990579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (26, 27, '987990579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (27, 28, '973990479', NULL, 'https://cursosinemweb.es/cursos-inem-cataluna/', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (28, 29, '941750579', NULL, 'http://www.larioja.org/npRioja/default/defaultpage.jsp?idtab=423414', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (29, 30, '982750779', NULL, 'http://traballo.xunta.es/cursos-dirixidos-a-traballadores-desempregados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (30, 31, '910504579', NULL, 'http://www.madrid.org/cs/Satellite?cid=1142337223387&language=es&pagename=Empleo%2FPage%2FEMPL_pintarContenidoFinal', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (31, 32, '952998679', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (32, 33, '968991579', NULL, 'http://www.sefcarm.es/web/pagina?IDCONTENIDO=30040&IDTIPO=100&RASTRO=c$m29962', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (33, 34, '948990579', NULL, 'http://www.navarra.es/home_es/Temas/Empleo+y+Economia/Empleo/Formacion/Personas+desempleadas/Default.htm', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (34, 35, '988750579', NULL, 'http://traballo.xunta.es/cursos-dirixidos-a-traballadores-desempregados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (35, 36, '979990579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (36, 37, '928990579', NULL, 'http://www3.gobiernodecanarias.org/empleo/portal/web/sce/servicios/cursos?inicio=false&q=&isla=&municipioCache=&modalidad=&sinMunicipio=true&tipoCurso=desempleados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (37, 38, '986981579', NULL, 'http://traballo.xunta.es/cursos-dirixidos-a-traballadores-desempregados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (38, 39, '923750579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (39, 40, '922990579', NULL, 'http://www3.gobiernodecanarias.org/empleo/portal/web/sce/servicios/cursos?inicio=false&q=&isla=&municipioCache=&modalidad=&sinMunicipio=true&tipoCurso=desempleados', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (40, 41, '921750579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (41, 42, '955563579', NULL, NULL, NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (42, 43, '975750579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (43, 44, '977990979', NULL, 'https://cursosinemweb.es/cursos-inem-cataluna/', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (44, 45, '978990579', NULL, 'http://plan.aragon.es/MapaRec.nsf/General', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (45, 46, '925990579', NULL, 'https://e-empleo.jccm.es/formacion/jsp/solicitudes/busquedaGrupos.jsp', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (46, 47, '963085779', NULL, 'http://www.ocupacio.gva.es:7017/portal/web/home/OfertaCursos2/BuscarCursos', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (47, 48, '983990979', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (48, 49, '980750579', NULL, 'http://www.empleo.jcyl.es/web/jcyl/Empleo/es/Plantilla66y33/1284326172615/_/_/_', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (49, 50, '976998979', NULL, 'http://plan.aragon.es/MapaRec.nsf/General', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (50, 51, '956984979', NULL, 'http://www.citapreviainem.es/cursos-inem-ceuta/', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (51, 52, '952990779', NULL, 'http://www.melillaorienta.es/?page=listadocursos', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.oficinas_sepe (id, id_provincia, telefono, web, url_cursos, url_orientacion, creado_en, actualizado_en) VALUES (52, 53, NULL, NULL, 'http://www.ocupacio.gva.es:7017/portal/web/home/OfertaCursos2/BuscarCursos', NULL, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');


--
-- Data for Name: prestaciones; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (1, 'Subsidio para mayores de 52 años', 'Ayuda hasta la jubilación para mayores de 52 que han agotado el paro. Cotiza para jubilación.', '{age:over_52,status:exhausted,income:low}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (2, 'Ayuda Familiar (Con Cargas)', 'Para quienes han agotado el paro y tienen responsabilidades familiares.', '{family:true,status:exhausted,income:low}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (3, 'Subsidio Mayores 45 sin cargas', 'Para mayores de 45 años que han agotado el paro y NO tienen cargas.', '{age:45_51,age:over_52,family:false,status:exhausted,income:low}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (4, 'Renta Activa de Inserción (RAI)', 'Para parados de larga duración (+12 meses), discapacidad o violencia de género.', '{"Tener menos de 65 años.","No tener ingresos propios superiores al 75% del SMI.","Ser desempleado de larga duración (más de 45 años).","O ser víctima de violencia de género, emigrante retornado o persona con discapacidad."}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (5, 'Subsidio Extraordinario (SED)', 'El último recurso si has agotado todas las demás ayudas.', '{"Haber agotado cualquier subsidio por desempleo.","Ser desempleado de larga duración inscrito antes del 01/05/2018.","Tener cargas familiares.","No haber percibido antes el PAE."}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (6, 'Subsidio Cotización Insuficiente', 'Si has trabajado menos de un año y no tienes derecho a paro.', '{status:never,income:low}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/no-he-trabajado-mas-de-un-ano.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');
INSERT INTO public.prestaciones (id, titulo, descripcion, requisitos, url, activo, creado_en, actualizado_en) VALUES (7, 'Víctimas Violencia Género', 'Programa de inserción y ayuda económica específica.', '{special:violence}', 'https://www.sepe.es/HomeSepe/Personas/distributiva-prestaciones/he-dejado-de-cobrar-el-paro.html', true, '2026-04-08 12:31:51.943828+00', '2026-04-08 12:31:51.943828+00');


--
-- Data for Name: reportes; Type: TABLE DATA; Schema: public; Owner: admin
--



--
-- Data for Name: votos; Type: TABLE DATA; Schema: public; Owner: admin
--

INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', 'a13d6d64-5ddc-4005-89b2-cbf738cebe43', -1, '2026-04-11 23:14:47.324296+00', '2026-04-11 23:14:48.276653+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', 'c9d9325b-889c-4e21-a957-62a040cb1515', 1, '2026-04-11 23:50:43.967963+00', '2026-04-11 23:50:43.967963+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', 'c99abdad-dbe1-4329-a3bd-31e7e20057e2', 1, '2026-04-12 00:33:36.903516+00', '2026-04-12 00:33:36.903516+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', '2073120f-24c0-4d3f-9f0c-f47773c663a2', -1, '2026-04-12 03:11:24.579092+00', '2026-04-12 03:11:24.579092+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', '5a35887f-11e6-4f31-a068-57c63d36c833', -1, '2026-04-12 03:16:55.10769+00', '2026-04-12 03:16:55.10769+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'curso', 'adadcae2-54d0-4e8c-97e5-6036ff5a7a7e', 1, '2026-04-12 03:11:23.405029+00', '2026-04-12 17:34:42.435241+00');
INSERT INTO public.votos (id_usuario, tipo_contenido, id_contenido, tipo_voto, creado_en, actualizado_en) VALUES ('d84b59ce-e85c-4e43-a65b-eb39ae3eb25e', 'oferta', '96650f3e-2121-49e8-b3f1-dcfb2b39243a', 1, '2026-04-13 13:23:36.82433+00', '2026-04-13 13:23:36.82433+00');


--
-- Name: comunidades_autonomas_id_seq; Type: SEQUENCE SET; Schema: public; Owner: admin
--

SELECT pg_catalog.setval('public.comunidades_autonomas_id_seq', 19, true);


--
-- Name: oficinas_sepe_id_seq; Type: SEQUENCE SET; Schema: public; Owner: admin
--

SELECT pg_catalog.setval('public.oficinas_sepe_id_seq', 52, true);


--
-- Name: prestaciones_id_seq; Type: SEQUENCE SET; Schema: public; Owner: admin
--

SELECT pg_catalog.setval('public.prestaciones_id_seq', 7, true);


--
-- PostgreSQL database dump complete
--

\unrestrict IqnPGtSBgodJkyoQehbSGECzFaXiW5JNTLAQ0Gehb7cZa3AwFt8HUPgZ6TvqbuR


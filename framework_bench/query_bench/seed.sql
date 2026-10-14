-- Seed data for experiment B, loaded by run.sh after the Prisma migrations (the schema of both
-- backends). One user and one project per worker (10), each project with 2 statuses and 100 tags,
-- half of them with a status, so the LEFT JOIN in tags_by_project returns both NULL and non-NULL
-- status columns. Fixed values and timestamps, so a fresh DB is the same for every run.

INSERT INTO public."User" (id, email, username, password, salt)
SELECT w, 'qbench' || w || '@bench.local', 'qbench_user_' || w, 'not-a-real-hash', 'not-a-real-salt'
FROM generate_series(1, 10) AS w;

INSERT INTO public."Project" (id, name, description, created_by_id, created_at, updated_at)
SELECT w, 'qbench project ' || w, 'seeded for experiment B', w,
       TIMESTAMP '2025-01-01 00:00:00', TIMESTAMP '2025-01-01 00:00:00'
FROM generate_series(1, 10) AS w;

INSERT INTO public."ProjectUser" (project_id, user_id, is_admin)
SELECT w, w, true FROM generate_series(1, 10) AS w;

INSERT INTO public."Status" (id, title, color_r, color_g, color_b, project_id, shape)
SELECT (w - 1) * 2 + s, 'status ' || s, 0.1 * s, 0.5, 1.0 - 0.1 * s, w,
       (CASE s WHEN 1 THEN 'Cuboid' ELSE 'Sphere' END)::"Shape"
FROM generate_series(1, 10) AS w, generate_series(1, 2) AS s;

INSERT INTO public."Tag" (id, title, project_id, position_x, position_y, position_z, created_by_id,
                          status_id, created_at, updated_at)
SELECT (w - 1) * 100 + t, 'seed tag ' || t, w, t * 0.25, t * 0.5, t * 0.75, w,
       CASE WHEN t % 2 = 0 THEN (w - 1) * 2 + 1 + (t / 2) % 2 END,
       TIMESTAMP '2025-01-01 00:00:00' + t * INTERVAL '1 minute',
       TIMESTAMP '2025-01-01 00:00:00' + t * INTERVAL '1 minute'
FROM generate_series(1, 10) AS w, generate_series(1, 100) AS t;

-- the explicit ids above don't advance the sequences
SELECT setval('"User_id_seq"', (SELECT max(id) FROM public."User"));
SELECT setval('"Project_id_seq"', (SELECT max(id) FROM public."Project"));
SELECT setval('"ProjectUser_id_seq"', (SELECT max(id) FROM public."ProjectUser"));
SELECT setval('"Status_id_seq"', (SELECT max(id) FROM public."Status"));
SELECT setval('"Tag_id_seq"', (SELECT max(id) FROM public."Tag"));

VACUUM ANALYZE;

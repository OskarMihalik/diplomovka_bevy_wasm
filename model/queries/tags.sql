--! select_tags (model_id, limit, offset)
SELECT * FROM public."Tag" where model_id = :model_id order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y)
INSERT INTO public."Tag"
(title, model_id, position_x, position_y, position_z)
VALUES(:title, :model_id, :position_x, :position_y, :position_z);

--! update_tag (id, title, position_x, position_y, position_z)
UPDATE public."Tag"
SET title=:title, position_x=:position_x, position_y=:position_y, position_z=:position_z
WHERE id=:id;

--! select_model (id, limit, offset)
SELECT id, "version", model_link, "name", created_at, updated_at, project_id
FROM public."Model" WHERE id=(:id) order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_model(version, model_link, name, project_id)
INSERT INTO public."Model"
("version", model_link, "name", project_id)
VALUES(:version, :model_link, :name, :project_id)
RETURNING id;

--! select_models(project_id, limit, offset)
SELECT id, "version", model_link, "name", created_at, updated_at, project_id
FROM public."Model"
WHERE project_id=(:project_id) order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! select_project (id)
SELECT id, "name", description, created_at, updated_at
FROM public."Project"
WHERE id=(:id);
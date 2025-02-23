--! select_tags (model_id, limit, offset)
SELECT * FROM public."Tag" where model_id = :model_id order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y, created_by_id)
INSERT INTO public."Tag"
(title, model_id, position_x, position_y, position_z, created_by_id)
VALUES(:title, :model_id, :position_x, :position_y, :position_z, :created_by_id);

--! update_tag (id, title, position_x, position_y, position_z)
UPDATE public."Tag"
SET title=:title, position_x=:position_x, position_y=:position_y, position_z=:position_z
WHERE id=:id;

--! select_model (id, limit, offset)
SELECT id, "version", model_link, "name", created_at, updated_at, project_id
FROM public."Model" WHERE id=(:id) order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_model(version, model_link, name, project_id, created_by_id)
INSERT INTO public."Model"
("version", model_link, "name", project_id, created_by_id)
VALUES(:version, :model_link, :name, :project_id, :created_by_id)
RETURNING id;

--! select_models(user_id, project_id, limit, offset)
SELECT model.id, model."version", model.model_link, model."name", model.created_at, model.updated_at, model.project_id
FROM public."Model" model
JOIN public."ProjectUser" projectUser ON model.project_id = projectUser.project_id
WHERE model.project_id=(:project_id) AND projectUser.user_id=(:user_id) order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! select_project (id)
SELECT id, "name", description, created_at, updated_at
FROM public."Project"
WHERE id=(:id);

--! select_projects (user_id, limit, offset)
SELECT DISTINCT project.id, project."name", project.description, project.created_at, project.updated_at
FROM public."Project" project
JOIN public."ProjectUser" projectUser ON project.id = projectUser.project_id
WHERE projectUser.user_id = (:user_id)
order by created_at DESC
LIMIT :limit
OFFSET :offset;

--! insert_project (name, description, created_by_id)
INSERT INTO public."Project" 
(name, description, created_by_id)
VALUES (:name, :description, :created_by_id)
RETURNING id;
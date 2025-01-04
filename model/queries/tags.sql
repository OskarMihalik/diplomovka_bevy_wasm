--! select_tags (model_id, limit, offset)
SELECT * FROM public."Tag" where model_id = :model_id order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y)
INSERT INTO public."Tag"
(title, model_id, position_x, position_y, position_z)
VALUES(:title, :model_id, :position_x, :position_y, :position_z);

--! select_model (id, limit, offset)
SELECT id, "version", model_link, "name", created_at, updated_at, project_id
FROM public."Model" WHERE id=(:id)order by created_at DESC 
LIMIT :limit
OFFSET :offset;
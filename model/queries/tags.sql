--! select_tags (limit, offset)
SELECT * FROM public."Tag" order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y)
INSERT INTO public."Tag"
(title, model_id, position_x, position_y, position_z)
VALUES(:title, :model_id, :position_x, :position_y, :position_z);
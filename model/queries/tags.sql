--! select_tags (limit, offset)
SELECT * FROM public.tag order by created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y)
INSERT INTO public.tag
(title, model_id, position_x, position_y, position_z)
VALUES(:title, :model_id, :position_x, :position_y, :position_z);
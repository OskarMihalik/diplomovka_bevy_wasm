
--! select_statuses (user_id, limit, offset, project_id)
select distinct  status.id, status.title, status.created_at, status.updated_at, status.color_r, status.color_g, status.color_b, status.project_id, status.shape
FROM public."Status" status
join public."ProjectUser" projectUser on projectUser.project_id = status.project_id 
where projectUser.user_id  = :user_id and projectuser.project_id = :project_id
LIMIT :limit
OFFSET :offset;

--! insert_status (title, color_r, color_g, color_b, project_id, shape)
INSERT INTO public."Status"
(title,  color_r, color_g, color_b, project_id, shape)
VALUES(:title, :color_r, :color_g, :color_b, :project_id, :shape)
RETURNING id;

--! update_status (id, title, color_r, color_g, color_b, project_id, shape)
UPDATE public."Status"
SET title=:title, color_r=:color_r, color_g=:color_g, color_b=:color_b, project_id=:project_id, shape=:shape
WHERE id=:id;

--! delete_status
DELETE FROM public."Status"
WHERE id = :id;


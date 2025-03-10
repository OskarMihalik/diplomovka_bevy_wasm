
--! select_statuses (user_id, limit, offset, project_id)
select distinct  status.id, status.title, status.created_at, status.updated_at, status.color_r, status.color_g, status.color_b, status.project_id
FROM public."Status" status
join public."Tag" tag on tag.status_id = status.id
join public."Model" model on model.id = tag.model_id 
join public."Project" project on project.id = model.project_id 
join public."ProjectUser" projectUser on projectuser.user_id = :user_id
WHERE project.id = :project_id
ORDER BY status.created_at DESC
LIMIT :limit
OFFSET :offset;

--! insert_status (title, color_r, color_g, color_b, project_id)
INSERT INTO public."Status"
(title,  color_r, color_g, color_b, project_id)
VALUES(:title, :color_r, :color_g, :color_b, :project_id)
RETURNING id;

--! update_status (id, title, color_r, color_g, color_b, project_id)
UPDATE public."Status"
SET title=:title, color_r=:color_r, color_g=:color_g, color_b=:color_b, project_id=:project_id
WHERE id=:id;

--! delete_status
UPDATE public."Tag"
SET status_id = NULL
WHERE status_id = :id;


--: Tag(id, email, username, title, model_id, position_x, position_y, position_z, created_by_id, created_at, status_title?, status_id?, status_color_r?, status_color_g?, status_color_b?, status_project_id?, shape?, scale_x, scale_y, scale_z, rotation_x, rotation_y, rotation_z)

--! select_tags (model_id, limit, offset) : Tag
SELECT DISTINCT tag.id, "user"."email", "user".username, tag.title, tag.model_id, tag.position_x, tag.position_y, tag.position_z, tag.created_by_id, tag.created_at,
"status".shape, tag.scale_x, tag.scale_y, tag.scale_z, tag.rotation_x, tag.rotation_y, tag.rotation_z,
"status".title as status_title, "status".id as status_id, "status".color_r as status_color_r, "status".color_g as status_color_g, "status".color_b as status_color_b, "status".project_id as status_project_id
FROM public."Tag" tag
JOIN public."User" "user" ON tag.created_by_id = "user".id
LEFT JOIN public."Status" "status" on tag.status_id = "status".id
where tag.model_id = :model_id
order by tag.created_at DESC 
LIMIT :limit
OFFSET :offset;

--! insert_tag (title, model_id, position_z, position_x, position_y, created_by_id)
INSERT INTO public."Tag"
(title, model_id, position_x, position_y, position_z, created_by_id)
VALUES(:title, :model_id, :position_x, :position_y, :position_z, :created_by_id);

--! update_tag (id, title, position_x, position_y, position_z, status_id?, scale_x, scale_y, scale_z, rotation_x, rotation_y, rotation_z)
UPDATE public."Tag"
SET title=:title, position_x=:position_x, position_y=:position_y, position_z=:position_z, status_id=:status_id, 
scale_x=:scale_x, scale_y=:scale_y, scale_z=:scale_z, rotation_x=:rotation_x, rotation_y=:rotation_y, rotation_z=:rotation_z
WHERE id=:id;

--! delete_tag
DELETE FROM public."Tag"
WHERE id = :id;

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


--! insert_tag_message (text, tag_id, created_by_id)
INSERT INTO public."TagMessage"
("text", tag_id, created_by_id)
VALUES(:text, :tag_id, :created_by_id);

--! select_tag_messages (tag_id, user_id, limit, offset)
SELECT DISTINCT tagMessage.id, tagMessage."text", tagMessage.created_at, tagMessage.updated_at, "user".username, "user".id as user_id, "user".email, projectUser.is_admin, tagMessage.created_by_id
FROM public."TagMessage" tagMessage
JOIN public."Tag" tag ON tagMessage.tag_id = tag.id
JOIN public."Model" model ON tag.model_id = model.id
JOIN public."Project" project ON model.project_id = project.id
JOIN public."ProjectUser" projectUser ON project.id = projectUser.project_id
JOIN public."User" "user" ON tagMessage.created_by_id = "user".id
WHERE tagMessage.tag_id = (:tag_id) AND projectUser.user_id = (:user_id)
order by tagMessage.created_at DESC
LIMIT :limit
OFFSET :offset;
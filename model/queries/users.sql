--! select_users(email, limit, offset)
SELECT id, "email", username, created_at, updated_at
FROM public."User"
WHERE email like :email
order by created_at DESC
LIMIT :limit
OFFSET :offset;

--! select_users_in_project(project_id, limit, offset)
SELECT u.id, u."email", u.username, u.created_at, u.updated_at, projectUser.is_admin
FROM public."User" as u
JOIN public."ProjectUser" as projectUser ON projectUser.user_id = u.id
WHERE projectUser.project_id = :project_id
order by created_at DESC
LIMIT :limit
OFFSET :offset;

--! select_user_by_id(id)
SELECT u.id, u."email", u.username, u.created_at, u.updated_at
FROM public."User" AS u
WHERE id = :id;

--! insert_project_user (project_id, user_id, is_admin)
INSERT INTO public."ProjectUser" 
(project_id, user_id, is_admin)
VALUES (:project_id, :user_id, :is_admin);

--! insert_user (email, username, password, salt)
INSERT INTO public."User"
(email, username, "password", salt)
VALUES(:email, :username, :password, :salt)
RETURNING id;

--! select_user (email)
SELECT id, email, username, password, salt  FROM public."User"
WHERE email=(:email);

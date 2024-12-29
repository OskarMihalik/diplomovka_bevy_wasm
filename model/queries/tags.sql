--! select_tags (limit, offset)
SELECT * FROM public.tag order by created_at DESC 
LIMIT :limit
OFFSET :offset;
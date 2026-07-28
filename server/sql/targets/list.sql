select count(*) over () as "total!",
       id,
       name,
       base_url,
       enabled,
       timeout_ms,
       note,
       created_at,
       updated_at
from targets
order by id desc
limit $1 offset $2

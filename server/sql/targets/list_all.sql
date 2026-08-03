select id,
       name,
       base_url,
       enabled,
       timeout_ms,
       note,
       created_at,
       updated_at
from targets
order by id asc

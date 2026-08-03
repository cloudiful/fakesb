select count(*) over () as "total!",
       id,
       matcher,
       target_id,
       action,
       response_template_id,
       delay_ms,
    sequence_mode,
       priority,
       enabled,
       note,
       created_at,
       updated_at
from rules
order by priority desc, id asc
limit $1 offset $2

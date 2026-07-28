select count(*) over () as "total!",
       id,
       service_code,
       message_type,
       message_code,
       target_id,
       mode,
       response_template_id,
       priority,
       enabled,
       note,
       created_at,
       updated_at
from rules
order by priority desc, id asc
limit $1 offset $2

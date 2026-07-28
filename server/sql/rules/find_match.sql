select id,
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
where enabled = true
  and service_code = $1
  and message_type = $2
  and message_code = $3
order by priority desc, id asc
limit 1

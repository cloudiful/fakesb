select id,
       matcher,
       target_id,
       action,
       response_template_id,
       priority,
       enabled,
       note,
       created_at,
       updated_at
from rules
where enabled = true
order by priority desc, id asc

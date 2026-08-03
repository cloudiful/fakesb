select id,
       name,
       content_type,
       raw_template,
       format,
       status_code,
       headers,
       enabled,
       note,
       created_at,
       updated_at
from response_templates
order by id asc

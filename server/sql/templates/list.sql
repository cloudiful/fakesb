select count(*) over () as "total!",
       id,
       name,
       content_type,
       raw_template,
       format,
       enabled,
       note,
       created_at,
       updated_at
from response_templates
order by id desc
limit $1 offset $2

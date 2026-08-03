select count(*)::bigint as "count!"
from rules
where response_template_id = $1

select count(*)::bigint as "count!"
from rules
where target_id = $1

select count(*)::bigint as "count!"
from rule_sequence_steps
where template_id = $1

select s.id,
       s.rule_id,
       s.step_index,
       s.template_id,
       t.name as template_name
from rule_sequence_steps s
join response_templates t on t.id = s.template_id
where s.rule_id = any($1)
order by s.rule_id asc, s.step_index asc

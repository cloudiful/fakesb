select l.id,
       l.occurred_at,
       l.rule_id,
       l.target_id,
       l.mode,
       l.service_code,
       l.message_type,
       l.message_code,
       l.http_status_code,
       l.ret_code,
       l.ret_msg,
       l.latency_ms,
       l.error_message,
       coalesce(
           json_agg(
               json_build_object(
                   'id', s.id,
                   'kind', s.kind,
                   'raw_body', s.raw_body,
                   'normalized_json', s.normalized_json
               ) order by s.id
           ) filter (where s.id is not null),
           '[]'::json
       ) as snapshots
from request_logs l
left join message_snapshots s on s.log_id = l.id
where l.id = $1
group by l.id

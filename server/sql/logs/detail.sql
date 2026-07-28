select l.id,
       l.occurred_at,
       l.rule_id,
       l.target_id,
       l.action,
       l.method,
       l.path,
       l.query_string,
       l.content_type,
       l.body_format,
       l.request_headers,
       l.response_headers,
       l.http_status_code,
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

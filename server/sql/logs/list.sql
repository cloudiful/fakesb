select count(*) over () as "total!",
       id,
       occurred_at,
       rule_id,
       target_id,
       action,
       method,
       path,
       query_string,
       content_type,
       body_format,
       request_headers,
       response_headers,
       http_status_code,
       latency_ms,
       error_message
from request_logs
where ($1::text is null or method = $1)
  and ($2::text is null or path = $2)
  and ($3::text is null or action = $3)
  and ($4::integer is null or http_status_code = $4)
  and ($5::timestamptz is null or occurred_at >= $5)
  and ($6::timestamptz is null or occurred_at <= $6)
order by occurred_at desc, id desc
limit $7 offset $8

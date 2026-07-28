select count(*) over () as "total!",
       id,
       occurred_at,
       rule_id,
       target_id,
       mode,
       service_code,
       message_type,
       message_code,
       http_status_code,
       ret_code,
       ret_msg,
       latency_ms,
       error_message
from request_logs
where ($1::text is null or service_code = $1)
  and ($2::text is null or message_type = $2)
  and ($3::text is null or message_code = $3)
  and ($4::text is null or mode = $4)
  and ($5::text is null or ret_code = $5)
  and ($6::timestamptz is null or occurred_at >= $6)
  and ($7::timestamptz is null or occurred_at <= $7)
order by occurred_at desc, id desc
limit $8 offset $9

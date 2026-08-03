delete from request_logs
where ($1::text is null or method = $1)
  and ($2::text is null or path = $2)
  and ($3::text is null or action = $3)
  and ($4::integer is null or http_status_code = $4)
  and ($5::timestamptz is null or occurred_at >= $5)
  and ($6::timestamptz is null or occurred_at <= $6)

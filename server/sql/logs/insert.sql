insert into request_logs(
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
)
values (now(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
returning id

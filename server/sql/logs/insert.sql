insert into request_logs(
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
)
values (now(), $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
returning id

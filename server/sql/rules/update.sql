update rules
set service_code = $2,
    message_type = $3,
    message_code = $4,
    target_id = $5,
    mode = $6,
    response_template_id = $7,
    priority = $8,
    enabled = $9,
    note = $10,
    updated_at = now()
where id = $1
returning id

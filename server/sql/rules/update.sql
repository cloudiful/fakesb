update rules
set matcher = $2,
    target_id = $3,
    action = $4,
    response_template_id = $5,
    delay_ms = $6,
    sequence_mode = $7,
    priority = $8,
    enabled = $9,
    note = $10,
    updated_at = now()
where id = $1
returning id

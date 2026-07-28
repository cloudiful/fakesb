update rules
set matcher = $2,
    target_id = $3,
    action = $4,
    response_template_id = $5,
    priority = $6,
    enabled = $7,
    note = $8,
    updated_at = now()
where id = $1
returning id

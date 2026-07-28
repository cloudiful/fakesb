insert into rules(
    service_code,
    message_type,
    message_code,
    target_id,
    mode,
    response_template_id,
    priority,
    enabled,
    note
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
returning id

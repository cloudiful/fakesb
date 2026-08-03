insert into rules(
    matcher,
    target_id,
    action,
    response_template_id,
    delay_ms,
    sequence_mode,
    priority,
    enabled,
    note
)
values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
returning id

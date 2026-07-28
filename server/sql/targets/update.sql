update targets
set name = $2,
    base_url = $3,
    enabled = $4,
    timeout_ms = $5,
    note = $6,
    updated_at = now()
where id = $1
returning id

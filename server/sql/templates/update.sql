update response_templates
set name = $2,
    content_type = $3,
    raw_template = $4,
    format = $5,
    status_code = $6,
    headers = $7,
    enabled = $8,
    note = $9,
    updated_at = now()
where id = $1
returning id

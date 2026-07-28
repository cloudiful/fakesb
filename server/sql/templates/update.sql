update response_templates
set name = $2,
    content_type = $3,
    raw_template = $4,
    format = $5,
    enabled = $6,
    note = $7,
    updated_at = now()
where id = $1
returning id

insert into response_templates(name, content_type, raw_template, format, enabled, note)
values ($1, $2, $3, $4, $5, $6)
returning id

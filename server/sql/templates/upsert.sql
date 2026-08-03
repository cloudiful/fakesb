insert into response_templates (name, content_type, raw_template, format, status_code, headers, enabled, note)
values ($1, $2, $3, $4, $5, $6, $7, $8)
on conflict (name) do update
set content_type = excluded.content_type,
    raw_template = excluded.raw_template,
    format = excluded.format,
    status_code = excluded.status_code,
    headers = excluded.headers,
    enabled = excluded.enabled,
    note = excluded.note,
    updated_at = now()
returning id

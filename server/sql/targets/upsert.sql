insert into targets (name, base_url, enabled, timeout_ms, note)
values ($1, $2, $3, $4, $5)
on conflict (base_url) do update
set name = excluded.name,
    enabled = excluded.enabled,
    timeout_ms = excluded.timeout_ms,
    note = excluded.note,
    updated_at = now()
returning id

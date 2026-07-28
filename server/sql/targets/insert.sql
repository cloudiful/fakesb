insert into targets(name, base_url, enabled, timeout_ms, note)
values ($1, $2, $3, $4, $5)
returning id

create table if not exists targets (
    id bigserial primary key,
    name text not null,
    base_url text not null unique,
    enabled boolean not null default true,
    timeout_ms integer not null default 10000,
    note text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table if not exists response_templates (
    id bigserial primary key,
    name text not null unique,
    content_type text not null default 'text/plain',
    raw_template text not null,
    format text not null default 'text' check (format in ('json', 'xml', 'text')),
    status_code integer not null default 200 check (status_code between 100 and 599),
    headers jsonb not null default '{}'::jsonb,
    enabled boolean not null default true,
    note text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table if not exists rules (
    id bigserial primary key,
    matcher jsonb not null default '{}'::jsonb,
    target_id bigint references targets(id),
    action text not null check (action in ('proxy', 'static')),
    response_template_id bigint references response_templates(id),
    priority integer not null default 0,
    enabled boolean not null default true,
    note text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index if not exists idx_rules_enabled_priority on rules(enabled, priority desc, id asc);

create table if not exists request_logs (
    id bigserial primary key,
    occurred_at timestamptz not null default now(),
    rule_id bigint references rules(id),
    target_id bigint references targets(id),
    action text check (action in ('proxy', 'static')),
    method text not null,
    path text not null,
    query_string text,
    content_type text,
    body_format text not null check (body_format in ('json', 'xml', 'text')),
    request_headers jsonb not null default '{}'::jsonb,
    response_headers jsonb not null default '{}'::jsonb,
    http_status_code integer,
    latency_ms bigint,
    error_message text
);

create index if not exists idx_request_logs_occurred_at on request_logs(occurred_at desc);
create index if not exists idx_request_logs_path on request_logs(path, occurred_at desc);

create table if not exists message_snapshots (
    id bigserial primary key,
    log_id bigint not null references request_logs(id) on delete cascade,
    kind text not null check (kind in ('request', 'response')),
    raw_body text not null,
    normalized_json jsonb not null default 'null'::jsonb
);

create index if not exists idx_message_snapshots_log_id on message_snapshots(log_id);

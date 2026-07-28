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
    content_type text not null default 'application/xml',
    raw_template text not null,
    format text not null default 'xml',
    enabled boolean not null default true,
    note text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table if not exists rules (
    id bigserial primary key,
    service_code text not null,
    message_type text not null,
    message_code text not null,
    target_id bigint references targets(id),
    mode text not null check (mode in ('passthrough', 'mock')),
    response_template_id bigint references response_templates(id),
    priority integer not null default 0,
    enabled boolean not null default true,
    note text,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index if not exists idx_rules_lookup
    on rules(service_code, message_type, message_code, enabled, priority desc, id asc);

create table if not exists request_logs (
    id bigserial primary key,
    occurred_at timestamptz not null default now(),
    rule_id bigint references rules(id),
    target_id bigint references targets(id),
    mode text check (mode in ('passthrough', 'mock')),
    service_code text not null,
    message_type text not null,
    message_code text not null,
    http_status_code text,
    ret_code text,
    ret_msg text,
    latency_ms bigint,
    error_message text
);

create index if not exists idx_request_logs_occurred_at on request_logs(occurred_at desc);
create index if not exists idx_request_logs_service on request_logs(service_code, message_type, message_code);

create table if not exists message_snapshots (
    id bigserial primary key,
    log_id bigint not null references request_logs(id) on delete cascade,
    kind text not null check (kind in ('request', 'response')),
    raw_body text not null,
    normalized_json jsonb not null default '{}'::jsonb
);

create index if not exists idx_message_snapshots_log_id on message_snapshots(log_id);

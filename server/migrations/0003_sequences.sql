alter table rules add column if not exists sequence_mode boolean not null default false;
alter table rules add column if not exists sequence_count bigint not null default 0;

create table if not exists rule_sequence_steps (
    id bigserial primary key,
    rule_id bigint not null references rules(id) on delete cascade,
    step_index integer not null,
    template_id bigint not null references response_templates(id),
    unique (rule_id, step_index)
);

create index if not exists idx_rule_sequence_steps_rule_id on rule_sequence_steps(rule_id);

alter table request_logs
    drop constraint if exists request_logs_rule_id_fkey,
    drop constraint if exists request_logs_target_id_fkey;

alter table request_logs
    add constraint request_logs_rule_id_fkey
        foreign key (rule_id) references rules(id) on delete set null,
    add constraint request_logs_target_id_fkey
        foreign key (target_id) references targets(id) on delete set null;

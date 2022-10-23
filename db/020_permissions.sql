create table permissions (
    id integer primary key generated always as identity,
    subject_table varchar not null,
    subject_id int not null,
    roll varchar not null,
    ability varchar not null,
    resource_table varchar,
    resource_id int,
    constraint unique_permissions unique (subject_table, subject_id, roll, ability, resource_table, resource_id)
);
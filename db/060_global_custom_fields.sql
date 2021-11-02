create table global_custom_fields (
    id integer primary key generated always as identity,

    name varchar not null,
    config json not null,

    comment varchar,

    constraint unique_name unique (name)
);
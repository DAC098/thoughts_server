create table global_custom_fields (
    id serial primary key,

    name varchar not null,
    config json not null,

    comment varchar,

    constraint unique_name unique (name)
);
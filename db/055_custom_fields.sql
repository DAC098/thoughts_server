create table custom_fields (
    id integer primary key generated always as identity,
    
    name varchar not null,
    owner integer not null,
    "order" integer default 0,
    issued_by integer,

    config json not null,

    comment varchar,

    constraint unique_name_owner unique (name, owner),
    constraint owner_fk foreign key (owner) references users (id),
    constraint issued_by_fk foreign key (issued_by) references users (id)
);
create table entries (
    id integer primary key generated always as identity,

    day timestamp with time zone not null default CURRENT_DATE,

    created timestamp with time zone not null default CURRENT_DATE,
    updated timestamp with time zone,
    deleted timestamp with time zone,

    owner integer not null,

    constraint unique_day_owner_key unique (day, owner),
    constraint owner_fk foreign key (owner) references users (id)
);

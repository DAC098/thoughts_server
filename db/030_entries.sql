create table entries (
    id serial primary key,

    day timestamp with time zone not null default CURRENT_DATE,

    owner integer not null,

    constraint unique_day_owner_key unique (day, owner),
    constraint owner_fk foreign key (owner) references users (id)
);
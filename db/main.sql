create database thoughts;

\connect thoughts;

create table users (
    id serial primary key,
    level integer not null,

    full_name varchar,

    public boolean not null default false,

    username varchar not null unique,
    hash varchar not null,

    email varchar unique not null
);

create table user_access_requests (
    owner integer not null,
    ability char(1) not null,
    allowed_for integer not null,
    expire timestamp with time zone default null,

    constraint owner_fk foreign key (owner) references users (id),
    constraint allowed_for_fk foreign key (allowed_for) references users (id),
    constraint unique_requests_per_user unique (owner, allowed_for),
    constraint not_same_user check (owner != allowed_for)
);

create table user_access (
    owner integer not null,
    ability char(1) not null,
    allowed_for integer not null,

    constraint owner_fk foreign key (owner) references users (id),
    constraint allowed_for_fk foreign key (allowed_for) references users (id),
    constraint unique_ability_per_user unique (owner, ability, allowed_for),
    constraint not_same_user check (owner != allowed_for)
);

create table user_sessions (
    token uuid not null,
    owner integer not null,

    constraint owner_fk foreign key (owner) references users (id)
);

create table entries (
    id serial primary key,

    day timestamp with time zone not null unique default CURRENT_DATE,

    owner integer not null,

    constraint owner_fk foreign key (owner) references users (id)
);

create table tags (
    id serial primary key,

    title varchar not null
);

create table entries2tags (
    id serial primary key,

    tag integer not null,

    entry integer not null,

    constraint unique_entry_tag unique (entry, tag),
    constraint entry_fk foreign key (entry) references entries (id),
    constraint tag_fk foreign key (tag) references tags (id)
);

create table text_entries (
    id serial primary key,

    thought text not null,

    private boolean default not null default false,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);

create table mood_fields (
    id serial primary key,
    
    name varchar not null,
    owner integer not null,
    issued_by integer

    config json not null,

    comment varchar,

    constraint owner_fk foreign key (owner) references users (id),
    constraint issued_by_fk foreign key (issued_by) references users (id),
    constraint unique_name_owner unique (name, owner)
);

create table mood_entries (
    id serial primary key,
    field integer not null,

    value json not null,

    comment varchar,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id),
    constraint field_fk foreign key (field) references mood_fields (id),
    constraint unique_entry_field unique (field, entry)
);
create user thoughts_user;

create database thoughts;

\connect thoughts;

create table users (
    id serial primary key,
    level integer not null,

    first_name varchar,
    last_name varchar,

    public boolean not null default false,

    username varchar not null unique,
    hash varchar not null,

    email varchar unique
);

create table user_access (
    owner integer not null,
    ability char(1) not null,
    for_user integer not null,

    constraint owner_fk foreign key (owner) references users (id),
    constraint for_user_fk foreign key (for_user) references users (id),
    constraint unique_ability_per_user unique (owner, ability, for_user),
    constraint not_same_user check (owner != for_user)
);

create table user_sessions (
    token uuid not null,
    owner integer not null,

    constraint owner_fk foreign key (owner) references users (id)
);

create table entries (
    id serial primary key,

    day date not null unique default CURRENT_DATE,

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

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);

create table mood_fields (
    id serial primary key,
    
    name varchar not null,
    owner integer not null,

    minimum integer,
    maximum integer,
    is_range boolean not null default false,

    comment varchar,

    constraint owner_fk foreign key (owner) references users (id),
    constraint unique_name_owner unique (name, owner)
);

create table mood_entries (
    id serial primary key,
    field integer not null,

    low integer not null default 0,
    high integer,

    comment varchar,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id),
    constraint field_fk foreign key (field) references mood_fields (id),
    constraint unique_entry_field unique (field, entry)
);
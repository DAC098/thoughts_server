create table users (
    id integer primary key generated always as identity,
    level integer not null,

    full_name varchar,

    public boolean not null default false,

    username varchar not null unique,
    hash varchar not null,

    email varchar unique
    email_verified boolean not null default false
);
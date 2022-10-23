create table groups (
    id integer primary key generated always as identity,
    name varchar not null unique
);
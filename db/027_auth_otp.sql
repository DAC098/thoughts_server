create table auth_otp (
    id integer primary key generated always as identity,
    users_id integer not null unique references users (id),
    algo smallint not null,
    secret bytea not null,
    digits smallint not null,
    step smallint not null,
    verified boolean not null default false
);
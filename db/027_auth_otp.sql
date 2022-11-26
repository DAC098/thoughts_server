create table auth_otp (
    id integer generated always as identity,
    users_id integer primary key,
    algo smallint not null,
    secret bytea not null,
    digits smallint not null,
    step smallint not null,
    verified boolean not null default false
)
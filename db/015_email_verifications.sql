create table email_verifications (
    owner integer not null primary key,
    key_id varchar not null unique,
    issued timestamp with time zone not null,
    constraint owner_fk foreign key (owner) references users (id)
);
create table user_sessions (
    token uuid not null,
    owner integer not null,
    
    expires timestamp with time zone not null,

    constraint owner_fk foreign key (owner) references users (id)
);
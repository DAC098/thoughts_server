create table user_sessions (
    token varchar primary key not null,
    owner integer not null,
    
    dropped boolean not null default false,

    issued_on timestamp with time zone not null,
    expires timestamp with time zone not null,

    verified boolean not null default false,
    use_csrf boolean not null default false,

    constraint owner_fk foreign key (owner) references users (id),
    constraint unique_user_and_token unique (owner, token)
);
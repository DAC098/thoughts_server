create table session_csrf_tokens (
    token varchar primary key not null,

    session_token varchar not null,

    issued_on timestamp with time zone not null,
    expires timestamp with time zone not null,

    constraint session_token_fk foreign key (session_token) references user_sessions (token)
);
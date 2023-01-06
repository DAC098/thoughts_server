create table auth_otp_codes (
    auth_otp_id integer not null references auth_otp (id),
    hash varchar not null,
    used boolean not null default false,
    constraint pk_auth_otp_hash primary key (auth_otp_id, hash),
    constraint unique_hash unique (hash)
);
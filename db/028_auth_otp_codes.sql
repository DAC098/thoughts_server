create table auth_otp_codes (
    auth_otp_id integer foreign key auth_otp(id),
    hash varchar not null,
    used boolean not null default false,
    add constraint unique_auth_otp_hash unique (auth_otp_id, hash)
)
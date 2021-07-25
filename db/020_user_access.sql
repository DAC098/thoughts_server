create table user_access (
    owner integer not null,
    ability char(1) not null,
    allowed_for integer not null,

    constraint owner_fk foreign key (owner) references users (id),
    constraint allowed_for_fk foreign key (allowed_for) references users (id),
    constraint unique_ability_per_user unique (owner, ability, allowed_for),
    constraint not_same_user check (owner != allowed_for)
);
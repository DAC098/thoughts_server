create table group_users (
    users_id integer references users (id),
    group_id integer references groups (id),
    primary key (users_id, group_id)
);
create table tags (
    id serial primary key,

    title varchar not null,
    
    owner integer not null,
    comment varchar,
    color varchar not null default '#ffffff',

    constraint unique_title_owner unique (title, owner),
    constraint owner_fk foreign key (owner) references users (id)
);
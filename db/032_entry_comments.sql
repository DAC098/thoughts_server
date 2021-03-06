create table entry_comments (
    id bigint primary key generated always as identity,
    
    entry integer not null,
    owner integer not null,

    comment varchar not null,
    created timestamp with time zone not null,
    updated timestamp with time zone,

    constraint entry_fk foreign key (entry) references entries (id),
    constraint field_fk foreign key (owner) references users (id)
);
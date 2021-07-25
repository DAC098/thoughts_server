create table entries2tags (
    id serial primary key,

    tag integer not null,

    entry integer not null,

    constraint unique_entry_tag unique (entry, tag),
    constraint entry_fk foreign key (entry) references entries (id),
    constraint tag_fk foreign key (tag) references tags (id)
);
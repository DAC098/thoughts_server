create table entry_markers (
    id serial primary key,
    title varchar not null,

    comment varchar,
    
    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);
create table entry_markers (
    id integer primary key generated always as identity,
    title varchar not null,

    comment varchar,
    
    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);
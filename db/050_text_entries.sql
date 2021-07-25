create table text_entries (
    id serial primary key,

    thought text not null,

    private boolean not null default false,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);
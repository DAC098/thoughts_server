create table text_entries (
    id integer primary key generated always as identity,

    thought text not null,

    private boolean not null default false,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);
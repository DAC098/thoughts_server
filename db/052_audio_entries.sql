create table audio_entries (
    id integer primary key generated always as identity,

    private boolean not null default false,

    comment varchar,

    entry integer not null,

    constraint entry_fk foreign key (entry) references entries (id)
);
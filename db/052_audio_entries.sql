create table audio_entries (
    id integer primary key generated always as identity,

    private boolean not null default false,

    comment varchar,

    entry integer not null,

    mime_type varchar not null,
    mime_subtype varchar not null,

    file_size bigint default 0,

    constraint entry_fk foreign key (entry) references entries (id)
);

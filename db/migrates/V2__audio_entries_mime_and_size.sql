alter table audio_entries
    add column mime_type varchar not null,
    add column mime_subtype varchar not null,
    add column file_size bigint default 0;

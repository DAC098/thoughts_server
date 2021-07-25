create table custom_field_entries (
    field integer not null,

    value json not null,

    comment varchar,

    entry integer not null,

    constraint entry_field_key primary key (field, entry),
    constraint entry_fk foreign key (entry) references entries (id),
    constraint field_fk foreign key (field) references custom_fields (id)
);
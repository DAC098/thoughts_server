alter table mood_fields rename to custom_fields;
alter table mood_entries rename to custom_field_entries;
alter table mood_fields_id_seq rename to custom_fields_id_seq;
alter table custom_field_entries 
    drop column id, 
    drop constraint unique_entry_field, 
    add constraint entry_field_key primary key (field, entry);
alter table custom_fields add column "order" integer default 0;

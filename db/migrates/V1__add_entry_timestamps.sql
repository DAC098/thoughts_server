alter table entries
    add column created timestamp with time zone not null default CURRENT_DATE,
    add column updated timestamp with time zone,
    add column deleted timestamp with time zone;

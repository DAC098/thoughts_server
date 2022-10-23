create table user_data (
    owner integer primary key,

    prefix varchar,
    suffix varchar,
    first_name varchar not null,
    last_name varchar not null,
    middle_name varchar,

    dob date not null,
    
    constraint owner_fk foreign key (owner) references users (id)
);
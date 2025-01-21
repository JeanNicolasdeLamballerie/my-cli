-- your sql goes here

create table master_user(
    id integer primary key not null,
    hash varchar not null unique
)

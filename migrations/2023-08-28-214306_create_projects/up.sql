create table projects (
    id integer primary key not null,
    name text not null unique,
    path text not null,
    language_id integer not null references languages(id)
)
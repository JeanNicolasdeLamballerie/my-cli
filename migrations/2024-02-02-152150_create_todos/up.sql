-- Your SQL goes here

CREATE TABLE todos (
    id integer not null PRIMARY KEY,
    title VARCHAR not null unique,
    subtitle text not null,
    content text not null,
    project_id integer not null references projects(id))

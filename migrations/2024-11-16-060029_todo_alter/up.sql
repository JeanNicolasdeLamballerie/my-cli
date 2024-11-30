CREATE TABLE todos (
    id integer not null PRIMARY KEY,
    title VARCHAR not null unique,
    subtitle text,
    content text,
    project_id integer not null references projects(id))

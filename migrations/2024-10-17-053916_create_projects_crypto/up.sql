-- Your SQL goes here
CREATE table projects_crypto_password(
    id integer unique primary key not null,
    crypto_data_id  integer not null references crypto_data(id),
    project_id integer not null references projects(id)
)

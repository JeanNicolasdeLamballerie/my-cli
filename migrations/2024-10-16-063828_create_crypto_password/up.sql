-- Your SQL goes here
CREATE table crypto_data(
    id integer unique primary key not null,
    encrypted text not null,
    host  text not null,
    nonce text not null)


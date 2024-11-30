CREATE TABLE todos_tags (
  id integer primary key not null,
  todo_id INTEGER not null REFERENCES todos(id),
  tag_id INTEGER not null REFERENCES tags(id)
)

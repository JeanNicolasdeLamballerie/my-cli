ALTER TABLE
    projects
ADD
    COLUMN script_id integer references scripts(id);

-- Your SQL goes here
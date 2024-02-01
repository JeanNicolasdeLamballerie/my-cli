// @generated automatically by Diesel CLI.

diesel::table! {
    languages (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    projects (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        language_id -> Integer,
        script_id -> Nullable<Integer>,
    }
}

diesel::table! {
    scripts (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        language_id -> Integer,
    }
}

diesel::table! {
    scripts_lang_defaults (id) {
        id -> Integer,
        language_id -> Integer,
        script_id -> Integer,
    }
}

diesel::table! {
    ssh (id) {
        id -> Integer,
        name -> Text,
        pw_name -> Text,
        user -> Text,
        host -> Text,
    }
}

diesel::table! {
    ssh_projects (id) {
        id -> Integer,
        project_id -> Integer,
        ssh_id -> Integer,
    }
}

diesel::joinable!(projects -> languages (language_id));
diesel::joinable!(projects -> scripts (script_id));
diesel::joinable!(scripts -> languages (language_id));
diesel::joinable!(scripts_lang_defaults -> languages (language_id));
diesel::joinable!(scripts_lang_defaults -> scripts (script_id));
diesel::joinable!(ssh_projects -> projects (project_id));
diesel::joinable!(ssh_projects -> ssh (ssh_id));

diesel::allow_tables_to_appear_in_same_query!(
    languages,
    projects,
    scripts,
    scripts_lang_defaults,
    ssh,
    ssh_projects,
);

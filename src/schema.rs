// @generated automatically by Diesel CLI.

diesel::table! {
    crypto_data (id) {
        id -> Integer,
        encrypted -> Text,
        host -> Text,
        nonce -> Text,
    }
}

diesel::table! {
    languages (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    master_user (id) {
        id -> Integer,
        hash -> Text,
    }
}

diesel::table! {
    projects (id) {
        id -> Integer,
        name -> Text,
        path -> Text,
        language_id -> Integer,
    }
}

diesel::table! {
    projects_crypto_password (id) {
        id -> Integer,
        crypto_data_id -> Integer,
        project_id -> Integer,
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

diesel::table! {
    tags (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    todos (id) {
        id -> Integer,
        title -> Text,
        subtitle -> Nullable<Text>,
        content -> Nullable<Text>,
        project_id -> Integer,
    }
}

diesel::table! {
    todos_tags (id) {
        id -> Integer,
        todo_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::joinable!(projects -> languages (language_id));
diesel::joinable!(projects_crypto_password -> crypto_data (crypto_data_id));
diesel::joinable!(projects_crypto_password -> projects (project_id));
diesel::joinable!(scripts -> languages (language_id));
diesel::joinable!(scripts_lang_defaults -> languages (language_id));
diesel::joinable!(scripts_lang_defaults -> scripts (script_id));
diesel::joinable!(ssh_projects -> projects (project_id));
diesel::joinable!(ssh_projects -> ssh (ssh_id));
diesel::joinable!(todos -> projects (project_id));
diesel::joinable!(todos_tags -> tags (tag_id));
diesel::joinable!(todos_tags -> todos (todo_id));

diesel::allow_tables_to_appear_in_same_query!(
    crypto_data,
    languages,
    master_user,
    projects,
    projects_crypto_password,
    scripts,
    scripts_lang_defaults,
    ssh,
    ssh_projects,
    tags,
    todos,
    todos_tags,
);

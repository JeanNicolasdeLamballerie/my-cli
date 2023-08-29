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
    }
}

diesel::joinable!(projects -> languages (language_id));

diesel::allow_tables_to_appear_in_same_query!(
    languages,
    projects,
);

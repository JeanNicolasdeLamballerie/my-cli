//use crate::schema::projects::dsl::*;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;
use std::path::Path;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();
    //todo dotenv for devs
    let mut database_path = match env::current_exe() {
        Ok(exe_path) => exe_path.to_owned(),
        Err(e) => panic!("failed to get current exe path: {e}"),
    };
    database_path.pop();
    database_path.push(Path::new("projects.db"));
    let database_url: &str = database_path.to_str().unwrap();
    // env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to database url..."))
}

use crate::models::{Language, NewLanguage};

use crate::schema::languages;
pub fn create_language(conn: &mut SqliteConnection, name: &str) -> Language {
    let new_language = NewLanguage { name };

    diesel::insert_into(languages::table)
        .values(&new_language)
        .returning(Language::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}
//todo add by name
pub fn fetch_projects(conn: &mut SqliteConnection, language: &str) -> Vec<Project> {
    match language {
        "all" | "a" => {
            return projects::table
                .select(Project::as_select())
                .load(conn)
                .expect("error handling language");
        }

        _ => {
            let l = get_language_by_name(conn, language);

            return projects::table
                .filter(projects::dsl::language_id.eq(l.id))
                .select(Project::as_select())
                .load(conn)
                .expect("error handling language");
        }
    }
}
pub fn fetch_single_project(conn: &mut SqliteConnection, name: &str) -> Project {
    return projects::table
        .filter(projects::dsl::name.eq(name))
        .select(Project::as_select())
        .load(conn)
        .expect("error handling language")
        .pop()
        .expect("the directory was not found");
}

pub fn fetch_languages(conn: &mut SqliteConnection, language: &str) -> Vec<Language> {
    match language {
        "all" | "a" => {
            return languages::table
                .select(Language::as_select())
                .load(conn)
                .expect("error handling language");
        }

        _ => {
            return languages::table
                .filter(languages::dsl::name.eq(language))
                .select(Language::as_select())
                .load(conn)
                .expect("error handling language");
        }
    }
}

use crate::models::{NewProject, Project};
use crate::schema::projects;

pub fn create_project(
    conn: &mut SqliteConnection,
    name: &str,
    path: &str,
    language: &str,
) -> Project {
    let language: Language = get_language_by_name(conn, language);
    let new_post = NewProject {
        name,
        path,
        language_id: &language.id,
    };

    diesel::insert_into(projects::table)
        .values(&new_post)
        .returning(Project::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn get_language_by_name(conn: &mut SqliteConnection, name: &str) -> Language {
    //  use crate::schema::languages;

    let mut languages: Vec<Language> = languages::table
        .filter(languages::dsl::name.eq(name))
        .select(Language::as_select())
        .load(conn)
        .expect("error handling language");
    match languages.pop() {
        Some(l) => l,
        None => Language {
            id: 1,
            name: "none".to_string(),
        },
    }
}

//use crate::schema::projects::dsl::*;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::path::Path;
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
pub const SEEDS_LAN: EmbeddedMigrations = embed_migrations!("./migrations/");
pub const SEEDS_LANG: EmbeddedMigrations = embed_migrations!("./seeds/language");
pub fn run_migration(conn: &mut SqliteConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap(); //.run_pending_migrations(MIGRATIONS).unwrap();
    let langs = fetch_languages(conn, "a");
    if !langs.iter().any(|language| language.name == "none") {
        // let has = conn.has_pending_migration(SEEDS_LANG).unwrap();
        conn.run_pending_migrations(SEEDS_LANG).unwrap();
    };
}

// #[derive(Debug)]
// pub struct ConnectionOptions {
//     pub enable_wal: bool,
//     pub enable_foreign_keys: bool,
//     pub busy_timeout: Option<Duration>,
// }
//
// impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
//     for ConnectionOptions
// {
//     fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
//         (|| {
//             if self.enable_wal {
//                 // conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
//             }
//             if self.enable_foreign_keys {
//                 conn.batch_execute("PRAGMA foreign_keys = ON;")?;
//             }
//             if let Some(d) = self.busy_timeout {
//                 conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
//             }
//             Ok(())
//         })()
//         .map_err(diesel::r2d2::Error::QueryError)
//     }
// }

pub fn generate_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    // TODO : remove unwraps
    let db = crate::config::db_path().unwrap();
    let db = db.to_str().unwrap();
    dbg!(db);
    let manager = diesel::r2d2::ConnectionManager::<SqliteConnection>::new(db);
    diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

// pub fn establish_connection() -> SqliteConnection {
//     dotenv().ok();
//     //todo dotenv for devs
//     let mut database_path = match env::current_exe() {
//         Ok(exe_path) => exe_path.to_owned(),
//         Err(e) => panic!("failed to get current exe path: {e}"),
//     };
//     database_path.pop();
//     database_path.push(Path::new("projects.db"));
//     let database_url: &str = database_path.to_str().unwrap();
//     // env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     SqliteConnection::establish(database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to database url..."))
// }

use crate::models::{
    CryptoData, Language, MasterUser, NewCryptoData, NewLanguage, NewMasterUser, NewStoredSsh,
    ProjectWithLanguageName, Ssh,
};

use crate::schema::{crypto_data, languages, master_user, ssh, ssh_projects};

pub enum CryptoFilterType {
    Host(String),
    Id(i32),
    All,
}

pub fn fetch_crypto(conn: &mut SqliteConnection, filter: CryptoFilterType) -> Vec<CryptoData> {
    match filter {
        CryptoFilterType::All => crypto_data::table
            .select(crypto_data::all_columns)
            .load(conn)
            .expect("error fetching the cryptos"),
        CryptoFilterType::Host(host) => crypto_data::table
            .select(crypto_data::all_columns)
            .filter(crypto_data::dsl::host.like(format!("{host}%")))
            .load(conn)
            .expect("error fetching the crypto host"),
        CryptoFilterType::Id(id) => crypto_data::table
            .select(crypto_data::all_columns)
            .filter(crypto_data::dsl::id.eq(id))
            .load(conn)
            .expect("error fetching the crypto id"),
    }
}
pub fn create_crypto(
    conn: &mut SqliteConnection,
    encrypted: &str,
    nonce: &str,
    host: &str,
) -> CryptoData {
    let new_crypto_data = NewCryptoData {
        encrypted,
        nonce,
        host,
    };
    diesel::insert_into(crypto_data::table)
        .values(&new_crypto_data)
        .returning(CryptoData::as_returning())
        .get_result(conn)
        .expect("Error saving new encrypted data...")
}

pub fn fetch_master_user(conn: &mut SqliteConnection) -> Option<MasterUser> {
    let u: Vec<MasterUser> = master_user::table
        .select(master_user::all_columns)
        .load(conn)
        .expect("Error requesting master user (db issue)");
    if u.len() > 1 {
        panic!("Your db is compromised. There is more than one super user.");
    }
    if u.is_empty() {
        return None;
    }
    Some(u[0].clone())
}

pub fn create_master_user(conn: &mut SqliteConnection, hash: &str) -> MasterUser {
    let new_master_user = NewMasterUser { hash };

    diesel::insert_into(master_user::table)
        .values(&new_master_user)
        .returning(MasterUser::as_returning())
        .get_result(conn)
        .expect("Error saving new master user")
}

pub fn create_language(conn: &mut SqliteConnection, name: &str) -> Language {
    let new_language = NewLanguage { name };

    diesel::insert_into(languages::table)
        .values(&new_language)
        .returning(Language::as_returning())
        .get_result(conn)
        .expect("Error saving new post")
}

pub fn single_project(conn: &mut SqliteConnection, project_name: &str) -> Project {
    projects::table
        .filter(projects::name.eq(project_name))
        .first(conn)
        .expect("The database request failed.")
}

//todo add by name
pub fn fetch_projects(
    conn: &mut SqliteConnection,
    searchterm: &str,
) -> Vec<ProjectWithLanguageName> {
    match searchterm {
        // "todo" | "todos" => {}
        "all" | "a" => {
            let projects_as_vec: Vec<(Project, String)> = projects::table
                .inner_join(languages::table)
                .select((projects::all_columns, languages::name))
                .load(conn)
                .expect("error handling language");
            projects_as_vec
                .iter()
                .map(|value| ProjectWithLanguageName::new(value.clone()))
                .collect()
        }
        term => {
            // let l = get_language_by_name(conn, searchterm);

            let projects_as_vec: Vec<(Project, String)> = projects::table
                .inner_join(languages::table)
                .select((projects::all_columns, languages::name))
                .filter(projects::dsl::name.like(format!("%{term}%")))
                .load(conn)
                .expect("error handling project");

            projects_as_vec
                .iter()
                .map(|value| ProjectWithLanguageName::new(value.clone()))
                .collect()
        }
    }
}
pub fn all_projects(conn: &mut SqliteConnection) -> Vec<ProjectWithLanguageName> {
    projects::table
        .inner_join(languages::table)
        .select((projects::all_columns, languages::name))
        .load(conn)
        .expect("error handling project")
        .iter()
        .map(|value: &(Project, String)| ProjectWithLanguageName::new(value.clone()))
        .collect()
    // s
}
pub fn fetch_single_project(conn: &mut SqliteConnection, name: &str) -> ProjectWithLanguageName {
    let projects_as_vec: Vec<(Project, String)> = projects::table
        .inner_join(languages::table)
        .select((projects::all_columns, languages::name))
        .filter(projects::dsl::name.eq(name))
        .load(conn)
        .expect("error handling language");
    let mut project_as_vec: Vec<ProjectWithLanguageName> = projects_as_vec
        .iter()
        .map(|value| ProjectWithLanguageName::new(value.clone()))
        .collect();
    project_as_vec.pop().expect("the directory was not found")
}
pub fn fetch_project_by_path(conn: &mut SqliteConnection, path: &str) -> ProjectWithLanguageName {
    let projects_as_vec: Vec<(Project, String)> = projects::table
        .inner_join(languages::table)
        .select((projects::all_columns, languages::name))
        .filter(projects::dsl::path.eq(path))
        .load(conn)
        .expect("error handling language");
    let mut project_as_vec: Vec<ProjectWithLanguageName> = projects_as_vec
        .iter()
        .map(|value| ProjectWithLanguageName::new(value.clone()))
        .collect();
    project_as_vec.pop().expect("the directory was not found")
}
pub fn fetch_languages(conn: &mut SqliteConnection, language: &str) -> Vec<Language> {
    match language {
        "all" | "a" => languages::table
            .select(Language::as_select())
            .load(conn)
            .expect("error handling language"),

        _ => languages::table
            .filter(languages::dsl::name.like(format!("%{language}%")))
            .select(Language::as_select())
            .load(conn)
            .expect("error handling language"),
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

pub fn alter_project_path(
    id: &i32,
    path: &Path,
    conn: &mut SqliteConnection,
) -> Result<Success, DatabaseError> {
    let resolved = std::path::absolute(path).unwrap();
    if let Err(err) = resolved.canonicalize() {
        return Err(DatabaseError::new(&err.to_string()));
    };
    // TODO: FIX these unwraps
    diesel::update(projects::table.filter(projects::dsl::id.eq(id)))
        .set(projects::dsl::path.eq(resolved.to_str().expect(
        "This is not a valid utf8 path. Contact the developer if you actually need this feature",
    ))).execute(conn).unwrap();
    Ok(Success::new(
        "Successfully edited the project's path.".into(),
        crate::ui::SuccessType::Database,
    ))
}
pub fn get_language(id: &i32, conn: &mut SqliteConnection) -> Language {
    languages::table
        .find(id)
        .first(conn)
        .expect("Invalid lang id")
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

use crate::models::{NewTodo, Todo, UpdateTodo};
use crate::schema::todos;
use crate::ui::{DatabaseError, Success};

/// True batch create. Returns the number of rows affected.
/// Should be updated with proper handling
pub fn batch_create_todo(todos: &Vec<NewTodo>, conn: &mut SqliteConnection) -> usize {
    if todos.is_empty() {
        return 0;
    }
    diesel::insert_into(todos::table)
        .values(todos)
        .execute(conn)
        .expect("error saving todo")
}

/// Edits a batch of todo; returns the number of todos updated. Each one generates a different
/// SQL request.
pub fn batch_edit_todo(todo_list: Vec<UpdateTodo>, conn: &mut SqliteConnection) -> usize {
    // crate::schema::todos::table.filter
    if todo_list.is_empty() {
        return 0;
    }
    for todo in todo_list.iter() {
        diesel::update(todos::table.filter(todos::id.eq(todo.id)))
            .set((
                todos::subtitle.eq(todo.subtitle),
                todos::content.eq(todo.content),
                todos::title.eq(todo.title),
            ))
            .execute(conn)
            .expect("error updating todo");
    }
    todo_list.len()

    // diesel::insert_into(todos::table)
    //     .values(&todos)
    //     .execute(&mut conn)
    //     .expect("error saving todo")
}
pub fn update_todo(todo: UpdateTodo, conn: &mut SqliteConnection) -> usize {
    // crate::schema::todos::table.filter
    diesel::update(todos::table.filter(todos::id.eq(todo.id)))
        .set((
            todos::subtitle.eq(todo.subtitle),
            todos::content.eq(todo.content),
            todos::title.eq(todo.title),
        ))
        .execute(conn)
        .expect("error updating todo")
}
// diesel::insert_into(todos::table)
//     .values(&todos)
//     .execute(&mut conn)
/// Inserts a single todo; returns it with it's Id.
pub fn create_todo(todo: NewTodo, conn: &mut SqliteConnection) -> Todo {
    diesel::insert_into(todos::table)
        .values(&todo)
        .returning(Todo::as_returning())
        .get_result(conn)
        .expect("error saving todo")
}

pub fn delete_all_todos(project_id: &i32, conn: &mut SqliteConnection) {
    diesel::delete(todos::table.filter(todos::project_id.eq(project_id)))
        .execute(conn)
        .unwrap_or_else(|_| (panic!("Error deleting todos related to project {}", project_id)));
}

/// Deletes a single todo for a given Todo ID.
pub fn delete_todo(id: &i32, conn: &mut SqliteConnection) {
    diesel::delete(todos::table.filter(todos::id.eq(id)))
        .execute(conn)
        .unwrap_or_else(|_| (panic!("Error deleting todo {}", id)));
}
/// Returns a single todo for a given Todo ID.
pub fn get_todo_id(id: i32, conn: &mut SqliteConnection) -> Todo {
    todos::table
        .filter(todos::dsl::id.eq(id))
        .select(Todo::as_select())
        .load(conn)
        .expect("error handling ssh")
        .pop()
        .expect("No Todo found with corresponding id")
}

pub fn get_todos_for_proj(project_id: i32, conn: &mut SqliteConnection) -> Vec<Todo> {
    todos::table
        .filter(todos::dsl::project_id.eq(project_id))
        .select(Todo::as_select())
        .load(conn)
        .expect("No Todos found...")
}

// TODO add a "hook" to hook a ssh to a project
pub fn create_ssh(
    conn: &mut SqliteConnection,
    name: &str,
    //  project_name: &str,
    pw_name: &str,
    user: &str,
    host: &str,
) -> Ssh {
    let new_ssh = NewStoredSsh {
        name,
        pw_name,
        user,
        host,
    };
    //   let project = fetch_single_project(conn, &project_name);
    // let ssh =
    diesel::insert_into(ssh::table)
        .values(&new_ssh)
        .returning(Ssh::as_returning())
        .get_result(conn)
        .expect("error saving ssh")
    // let new_proj_ssh = NewProjectSsh {
    //     ssh_id: &ssh.id,
    //     project_id: &project.id,
    // };
    // diesel::insert_into(ssh_projects::table)
    //     .values(&new_proj_ssh)
    //     .returning(SSHProjects::as_returning())
    //     .get_result(conn)
    //     .expect("error inserting in ssh+proj");

    // ssh
}

pub fn get_ssh(conn: &mut SqliteConnection, ssh_name: &str) -> Ssh {
    ssh::table
        .filter(ssh::dsl::name.eq(ssh_name))
        .select(Ssh::as_select())
        .load(conn)
        .expect("error handling ssh")
        .pop()
        .expect("No SSH found")
}
pub fn get_ssh_by_project(conn: &mut SqliteConnection, project_name: &str) -> Vec<Ssh> {
    projects::table
        .inner_join(ssh_projects::table.inner_join(ssh::table))
        .filter(projects::name.eq(project_name))
        .select(ssh::all_columns)
        .load::<Ssh>(conn)
        .unwrap()
}

pub fn get_script(_conn: &mut SqliteConnection, _name: Option<&str>) {}

pub trait Save<S> {
    fn save_to_db(
        &mut self,
        conn: &mut SqliteConnection,
    ) -> Result<crate::ui::Success, crate::ui::DatabaseError>;
    fn to_saved_format(&mut self) -> S;
}

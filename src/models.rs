use diesel::{prelude::*, sql_types::Nullable};
use tabled::Tabled;

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- Projects ------------------------------
// --------------------------------------------------------
// --------------------------------------------------------

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::projects)]
#[diesel(belongs_to(Language))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub language_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::projects)]
pub struct NewProject<'a> {
    pub name: &'a str,
    pub path: &'a str,
    pub language_id: &'a i32,
}

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- Languages -----------------------------
// --------------------------------------------------------
// --------------------------------------------------------

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::languages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Language {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::languages)]
pub struct NewLanguage<'a> {
    pub name: &'a str,
}

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- Projects + Lang------------------------
// --------------------------------------------------------
// --------------------------------------------------------

#[derive(Tabled, Clone, PartialEq)]
pub struct ProjectWithLanguageName {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub language_name: String,
}

impl ProjectWithLanguageName {
    pub fn new((project, language_name): (Project, String)) -> ProjectWithLanguageName {
        ProjectWithLanguageName {
            id: project.id,
            name: project.name,
            path: project.path,
            language_name,
        }
    }
}

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- Encrypted data ------------------------
// --------------------------------------------------------
// --------------------------------------------------------

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::master_user)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MasterUser {
    pub id: i32,
    pub hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::master_user)]
pub struct NewMasterUser<'a> {
    pub hash: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::projects_crypto_password)]
#[diesel(belongs_to(Project))]
#[diesel(belongs_to(CryptoData))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ProjectsCryptoData {
    pub id: i32,
    pub crypto_data_id: i32,
    pub project_id: i32,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::crypto_data)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CryptoData {
    pub id: i32,
    pub encrypted: String,
    pub host: String,
    pub nonce: String,
}
#[derive(Insertable)]
#[diesel(table_name = crate::schema::crypto_data)]
pub struct NewCryptoData<'a> {
    pub encrypted: &'a str,
    pub host: &'a str,
    pub nonce: &'a str,
}

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- TODOS ---------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
#[derive(Tabled)]
pub struct FormattedTodo {
    pub id: i32,
    pub title: String,
    pub subtitle: String,
    pub content: String,
    pub project_id: i32,
    pub new: bool,
}
impl From<Todo> for FormattedTodo {
    fn from(value: Todo) -> Self {
        Self {
            id: value.id,
            title: value.title,
            subtitle: value.subtitle.unwrap_or(String::from("None")),
            content: value.content.unwrap_or(String::from("None")),
            project_id: value.project_id,
            new: false,
        }
    }
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(belongs_to(Project))]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub content: Option<String>,
    pub project_id: i32,
}
#[derive(Insertable)]
#[diesel(table_name = crate::schema::todos)]
pub struct NewTodo<'a> {
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub content: Option<&'a str>,
    pub project_id: &'a i32,
}

pub struct UpdateTodo<'a> {
    pub id: &'a i32,
    pub title: &'a str,
    pub subtitle: Option<&'a str>,
    pub content: Option<&'a str>,
    pub project_id: &'a i32,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Tag {
    pub id: i32,
    pub name: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tags)]
pub struct NewTag<'a> {
    pub name: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(belongs_to(Tag))]
#[diesel(belongs_to(Todo))]
#[diesel(table_name = crate::schema::todos_tags)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct TodoTag {
    pub id: i32,
    pub tag_id: i32,
    pub todo_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::todos_tags)]
pub struct NewTodoTag<'a> {
    pub tag_id: &'a i32,
    pub todo_id: &'a i32,
}

// --------------------------------------------------------
// --------------------------------------------------------
// --------------------------------------------------------
// ---------------- SSH -----------------------------------
// --------------------------------------------------------
// --------------------------------------------------------

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::ssh)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Ssh {
    pub id: i32,
    pub name: String,
    pub pw_name: String,
    pub host: String,
    pub user: String,
}

//#[diesel(belongs_to(Language))]
#[derive(Insertable)]
#[diesel(table_name = crate::schema::ssh)]
pub struct NewStoredSsh<'a> {
    pub name: &'a str,
    pub pw_name: &'a str,
    pub host: &'a str,
    pub user: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, Associations, PartialEq, Debug, Clone, Tabled)]
#[diesel(table_name = crate::schema::ssh_projects)]
#[diesel(belongs_to(Project))]
#[diesel(belongs_to(Ssh))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct SSHProjects {
    pub id: i32,
    pub ssh_id: i32,
    pub project_id: i32,
}
#[derive(Insertable)]
#[diesel(table_name = crate::schema::ssh_projects)]
pub struct NewProjectSsh<'a> {
    pub ssh_id: &'a i32,
    pub project_id: &'a i32,
}

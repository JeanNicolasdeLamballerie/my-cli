use diesel::prelude::*;
use tabled::Tabled;
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

// #[derive(Insertable)]
// #[diesel(table_name = crate::schema::projects)]
// pub struct NewProject<'a> {
//     pub name: &'a str,
//     pub path: &'a str,
//     pub language_id: &'a i32,
// }
//
//
//
//

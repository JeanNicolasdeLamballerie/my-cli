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

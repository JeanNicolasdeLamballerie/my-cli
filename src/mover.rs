// use clap::Args;
//
// use crate::database;

use diesel::SqliteConnection;

use crate::database;
//use std::io;
pub fn move_to(conn: &mut SqliteConnection, name: &Option<String>) {
    let project = match name {
        Some(n) => database::fetch_single_project(conn, n),
        None => panic!("no name found"),
    };
    println!("{}", project.path);
}

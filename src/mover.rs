use crate::database::{self, establish_connection};
//use std::io;
pub fn move_to(name: &Option<String>) {
    let mut conn = establish_connection();
    let project = match name {
        Some(n) => database::fetch_single_project(&mut conn, n),
        None => panic!("no name found"),
    };
    println!("{}", project.path);
}

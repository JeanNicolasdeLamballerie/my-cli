use std::process::Command;

use diesel::SqliteConnection;

use crate::database::fetch_single_project;

pub fn run_command(conn: &mut SqliteConnection, name: &Option<String>, command: &String) {
    println!("{:?}, {:?}", name, command);
    let mut user_command = Command::new(command);
    match name {
        Some(project) => {
            let prj = fetch_single_project(conn, project);
            user_command.current_dir(prj.path);
        }
        None => (),
    }
    user_command.spawn().expect("error in spawning the process");
    //.arg("--help").spawn().expect("err");
}

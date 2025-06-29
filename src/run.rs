use std::{env::current_dir, process::Command};

use diesel::SqliteConnection;

use crate::{
    database::{fetch_project_by_path, fetch_single_project},
    tcp_println,
};

pub fn run_command(conn: &mut SqliteConnection, name: &Option<String>, command: &Option<String>) {
    tcp_println!("{:?}, {:?}", name, command);
    match command {
        None => (),
        Some(_string) => {
            let mut _user_command = Command::new(command.as_ref().unwrap());
            match name {
                Some(string) => {
                    tcp_println!("{}", string);
                    tcp_println!("project X being used");
                    // let proj = fetch_single_project(conn, string);

                    todo!("fetch all commands");
                }
                None => {
                    tcp_println!("No path, assuming the current directory as project directory");
                    let path = current_dir();
                    tcp_println!("path : {:?}", path);
                    let _proj = fetch_project_by_path(conn, path.unwrap().to_str().unwrap());
                    todo!();
                }
            }
        }
    }

    // if name == &None && command == &None {}
    let mut user_command = Command::new(command.as_ref().unwrap());
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

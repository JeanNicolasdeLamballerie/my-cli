use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
    thread::park_timeout,
    time::Duration,
};

use diesel::SqliteConnection;

use crate::{
    database::{create_ssh, get_ssh},
    logger::{print, TablingOptionsBuilder},
};

pub fn ssh_into(
    conn: &mut SqliteConnection,
    new: &Option<String>,
    name: &String,
    host: &Option<String>,
    user: &Option<String>,
    settings: &mut TablingOptionsBuilder,
) {
    let ssh = match new {
        Some(pw_name) => {
            let user_str = user.clone().unwrap();
            let host_str = host.clone().unwrap();
            create_ssh(conn, &name, &pw_name, &user_str, &host_str)
        }
        None => get_ssh(conn, &name), //  let mut project = fetch_single_project(conn, &name);
                                      // UserOrganization::belonging_to(&organizations)
                                      // .inner_join(user::table)
    };
    let mut table = tabled::Table::new(vec![ssh.clone()]);
    print(&mut table, settings);
    if new.is_some() {
        return;
    }
    let password = get_password(&ssh.pw_name);
    let user = ssh.user;
    let host = ssh.host;
    let ssh_args = format!("{user}@{host}");
    let mut handle = Command::new("ssh")
        .arg("-t")
        .arg("-t")
        .arg(ssh_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    println!("{:?}", handle);
    println!("password : {}", password);
    park_timeout(Duration::from_millis(5000));
    handle
        .stdin
        .as_ref()
        .unwrap()
        .write(&password.as_bytes())
        .unwrap();
    //  println!("wrote password");
    if let Some(ref mut stdout) = handle.stdout {
        for _line in BufReader::new(stdout).lines() {}
        handle.wait().unwrap();
    }
    // let out = handle.stdout.unwrap();
    // for line in io::stdou {
    //     println!("in : {:?}", line);
    // }

    // for line in io::stdin().lines() {
    //     println!("in : {:?}", line);
    // }
    //  let mut stdin = handle.stdin.take().unwrap();
    //   loop {
    // for line in BufReader::new(out).lines() {
    //     let mut l: String = line.unwrap();
    //     let eol: &str = "\n";
    //     l = l + eol;
    //     if l.contains("Could not resolve") {
    //         continue;
    //     }
    //     if l.contains("password") {
    // match &stdin.write_fmt(format_args!("{}", password)) {
    //     Ok(()) => (),
    //     Err(_error) => panic!("Password not written"),
    // };
    //     }
    // }
    //   }
    // let user = users::table
    //     .find(user_id)
    //     .first::<User>(conn)?;

    // UserOrganization::belonging_to(&user)
    //     .inner_join(organizations::table)
    //     .select(organizations::all_columns)
    //     .load::<Organization>(conn)?;
}

fn get_password(id: &str) -> String {
    match Command::new("dcli")
        .arg("p")
        .arg("-o")
        .arg("password")
        .arg(id)
        .output()
    {
        Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Err(error) => panic!("{}", error),
    }
}

//todo : traits...
//
//
// struct SshSolution {
//   handle() -> String,
// }

use clap::{Parser, Subcommand};
use my_cli::database::run_migration;
use my_cli::ssh::ssh_into;
use my_cli::{
    database::{
        create_language, create_project, establish_connection, fetch_languages, fetch_projects,
    },
    logger::{self, print},
    mover,
    run::run_command,
};
use resolve_path::PathResolveExt;
use std::time::SystemTime;
use tabled::Table;
/// 󰉊 Blazing fast project manager CLI 󰉊
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    //   #[arg(optional = true)]
    // path: String,
}

#[derive(Subcommand, Debug)]
enum TypeOfAdds {
    /// (add) Language
    L { language_name: String },
    /// (add) Project
    P {
        project_path: String,
        project_name: String,
        ///The language to register the project with
        #[arg(short, long)]
        language: String,
    },
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// [EXPERIMENTAL] SSH into a server. Uses dashline cli.
    Ssh {
        #[arg(short, long)]
        new: Option<String>,
        name: String,
        user: Option<String>,
        host: Option<String>,
    },
    /// Move to specific project
    Move {
        name: Option<String>,
        //
        // /// Name of the person to greet
        // #[arg(short, long)]
        // name: String,
        //
    },
    /// Add languages & projects
    Add {
        #[command(subcommand)]
        add_type: TypeOfAdds,
        // path: String,
        // name: String,
        // #[arg(short, long)]
        // language: Option<String>,
    },
    /// Show languages & projects
    Show {
        language: String,
        ///Indicate whether you wanna show languages or projects.
        #[arg(short, long, default_value_t = false)]
        lang_query: bool,
    },
    /// [EXPERIMENTAL] Run a script (associated with a project)
    Run {
        //NOTE : This order is important, setting command before name allows name to become a
        //positional argument
        #[arg(index = 0)]
        name: Option<String>,
        /// If not present, defaults to current directory.
        #[arg(num_args(0..))]
        command: Option<String>,
    },
}
// TODO : Fix that mess for run & SSH. Disabled for now.
fn parse() {
    // let dt_start = chrono::Utc::now();
    let timestamp_start = SystemTime::now();
    let cli = Cli::parse();
    let mut default_settings = logger::TablingOptionsBuilder::default();
    let settings = default_settings
        .color(Some(String::from("default")))
        .first_row_color(Some(String::from("blue")))
        .first_col_color(Some(String::from("magenta")))
        .header(Some(String::from("Query result :")));
    let mut conn = establish_connection();
    run_migration(&mut conn);
    match &cli.command {
        // Commands::Ssh {
        //     new,
        //     name,
        //     user,
        //     host,
        //     // TODO : Generate the struct here. Messing up the argument order is too easy
        // } => ssh_into(&mut conn, new, name, host, user, settings),
        Commands::Move { name } => mover::move_to(&mut conn, name),
        // Commands::Run { name, command } => run_command(&mut conn, name, command),
        Commands::Add { add_type } => match &add_type {
            TypeOfAdds::L { language_name } => {
                let lg = create_language(&mut conn, language_name);
                let mut table = tabled::Table::new(vec![lg.clone()]);
                print(&mut table, settings);
            }
            TypeOfAdds::P {
                project_path,
                project_name,
                language,
            } => {
                let path_item = project_path.resolve();
                match path_item.to_str() {
                    Some(val) => {
                        let prj = create_project(&mut conn, project_name, val, language);
                        let mut table = tabled::Table::new(vec![prj.clone()]);
                        print(&mut table, settings);
                    }
                    None => {
                        panic!("No valid path");
                    }
                }
            }
        },
        Commands::Show {
            language,
            lang_query,
        } => {
            if *lang_query {
                let languages = fetch_languages(&mut conn, language);
                let length = languages.len();
                let mut table = Table::new(languages);
                print(
                    &mut table,
                    settings.clone().header(Some(
                        format!("Language query result : {} language(s)", length).to_string(),
                    )),
                );
            } else {
                let projects = fetch_projects(&mut conn, language);
                let length = projects.len();
                let mut table = Table::new(projects);
                print(
                    &mut table,
                    settings.clone().header(Some(
                        format!("Project query result : {} project(s)", length).to_string(),
                    )),
                );
            }
        }
        _ => println!("Unsupported command..."),
    }

    //  let dt_end = chrono::Utc::now();
    let timestamp_end = SystemTime::now();
    let duration = timestamp_end.duration_since(timestamp_start).unwrap();
    match &cli.command {
        Commands::Move { name: _ } => (),
        _ => println!("Total query time : {} ms.", (duration.as_millis())),
    };
}

fn main() {
    parse();
}

// struct Foo {
//     message: String,
// }

// fn get_foo(message: &str) -> Foo {
//     Foo {
//         message: String::from(message),
//     }
// }

// fn use_foo(foo: Foo) {
//     println!("{}", foo.message);
// }
// fn main() {
//     let foo: Foo = Foo {
//         message: String::from("hello world"),
//     };

//     let some_foo = Foo {
//         message: String::from("hello world"),
//     };

//     let some_other_foo = get_foo("hello world");
//     use_foo(foo);
//     use_foo(some_foo);
//     use_foo(some_other_foo);
// }

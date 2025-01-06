use clap::{Parser, Subcommand};
use egui::mutex::Mutex;
use my_cli::database::{all_projects, run_migration, CryptoFilterType};
// use my_cli::ssh::ssh_into;
use my_cli::auth;
use my_cli::editor::TodoEditor;
use my_cli::exceptions::{Action, HandleException, Warning};
use my_cli::models::{Project, ProjectWithLanguageName};
use my_cli::todos::{TodoId, TodoList};
use my_cli::{
    database::{
        create_language, create_project, establish_connection, fetch_languages, fetch_projects,
    },
    logger::{self, print},
    mover,
};
use resolve_path::PathResolveExt;
use std::env::current_dir;
use std::path::PathBuf;
use std::sync::Arc;
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
    Lang { language_name: String },
    /// (add) Project
    Project {
        project_path: String,
        project_name: String,
        ///The language to register the project with
        #[arg(short, long)]
        language: String,
    },
    /// (add) Todo to project
    Todo {
        ///Your new todo's title
        todo_title: String,
        ///Path to the matching project.
        path: Option<String>,
        //TODO add tags ?
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
    Retrieve {
        #[arg(long)]
        host: String,
    },
    /// Store a key-value. Requires master password.
    Store {
        #[arg(long)]
        host: String,
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
        searchterm: String,
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
fn parse(warnings: Arc<Mutex<Vec<Warning>>>) {
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
        Commands::Retrieve { host } => {
            let key = auth::manager::requires_password(&mut conn);
            let cleartext =
                auth::manager::retrieve_encrypted(&key, CryptoFilterType::Host(host.to_owned()));
            auth::manager::show_password(&cleartext).unwrap();
        }
        Commands::Store { host } => {
            println!("our host : {host}");
            let key = auth::manager::requires_password(&mut conn);
            let data = auth::manager::hidden_user_input(0);
            auth::manager::store_encrypted(&data, host, &key);
            let _cleartext =
                auth::manager::retrieve_encrypted(&key, CryptoFilterType::Host(host.to_owned()));
        }
        // Commands::Ssh {
        //     new,
        //     name,
        //     user,
        //     host,
        //     // TODO : Generate the struct here. Messing up the argument order is too easy
        // } => my_cli::ssh::ssh_into(&mut conn, new, name, host, user, settings),
        // Commands::Move { name } => mover::move_to(&mut conn, name),
        // Commands::Run { name, command } => run_command(&mut conn, name, command),
        Commands::Add { add_type } => {
            match &add_type {
                TypeOfAdds::Lang { language_name } => {
                    let lg = create_language(&mut conn, language_name);
                    let mut table = tabled::Table::new(vec![lg.clone()]);
                    print(&mut table, settings);
                }
                TypeOfAdds::Project {
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
                TypeOfAdds::Todo { todo_title, path } => {
                    let current = std::env::current_dir().unwrap();
                    let mut target_proj: Option<&ProjectWithLanguageName> = None;
                    let target = match path {
                        //TODO handle errors
                        Some(path) => match path.try_resolve() {
                            Ok(p) => p
                                .to_path_buf()
                                .canonicalize()
                                .expect("The path should support being canonicalized."),
                            Err(err) => {
                                println!("Err ----------- \n {}", err.to_string());
                                panic!("An error occured. The path provided is invalid.");
                            }
                        },
                        None => current.canonicalize().unwrap(),
                    };
                    let mut projects = all_projects();
                    for project in projects.iter_mut() {
                        //FIXME unsafe unwrap
                        let p = PathBuf::from(&project.path).canonicalize();
                        match p {
                            Ok(v) => {
                                if target.eq(&v) {
                                    target_proj = Some(project);
                                    //FIXME we probably shouldn't break, but instead continue and check for similar values (e.g we're one level deep into the file system)
                                    break;
                                }
                            }
                            //TODO can do better error handling here
                            Err(_err) => {
                                let msg = format!("The path {} cannot be found. Consider removing or disabling the project ({}) ...", &project.path, project.name);
                                warnings.lock().push(Warning::new(&msg, true))
                            }
                        };
                    }
                    if let Some(project) = target_proj {
                        let options = eframe::NativeOptions {
                            viewport: egui::ViewportBuilder::default().with_maximized(true), //.with_inner_size([320.0, 240.0])
                            ..Default::default()
                        };

                        let mut list: TodoList = TodoList::default();
                        list.with_parent(project);
                        list.retrieve();
                        let todo =
                            TodoEditor::new("md", todo_title, "", "", TodoId::New(0), project.id);

                        list.add(todo);
                        eframe::run_native(
                            "Your Todos",
                            options,
                            Box::new(|_cc| {
                                // This gives us image support:
                                // egui_extras::install_image_loaders(&cc.egui_ctx);

                                Ok(Box::new(list))
                            }),
                        )
                        .unwrap();
                    }
                }
            }
        }
        Commands::Show {
            searchterm,
            lang_query,
        } => {
            if *lang_query {
                let languages = fetch_languages(&mut conn, searchterm);
                let length = languages.len();
                let mut table = Table::new(languages);
                print(
                    &mut table,
                    settings.clone().header(Some(
                        format!("Language query result : {} language(s)", length).to_string(),
                    )),
                );
            } else {
                let projects = fetch_projects(&mut conn, searchterm);
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
    let warnings: Arc<Mutex<Vec<Warning>>> = Arc::new(Mutex::new(Vec::new()));
    // view();
    parse(warnings.clone());
    for warning in warnings.lock().iter() {
        warning.warn();
    }
}

fn view() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(), //.with_inner_size([320.0, 240.0])
        ..Default::default()
    };
    eframe::run_native(
        "Your Todos",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(TodoList::default()))
        }),
    )
    .unwrap();
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

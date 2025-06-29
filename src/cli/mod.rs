use crate::database::{all_projects, run_migration, single_project, CryptoFilterType};
use crate::server::daemon::{DaemonState, TCP_OUTPUT};
use crate::server::spawn;
use clap::{Parser, Subcommand};
use diesel::SqliteConnection;
use egui::mutex::Mutex;
use log::trace;
// use my_cli::ssh::ssh_into;
use crate::editor::TodoEditor;
use crate::exceptions::Warning;
use crate::fonts::FONTS;
use crate::{auth, tcp_println};
// use crate::logger::setup_logger;
use crate::models::ProjectWithLanguageName;
use crate::mover::move_to;
use crate::projects_ui::{ProjectEditor, ProjectViewer};
use crate::todos::{StoredId, TodoList};
use crate::{
    database::{create_language, create_project, fetch_languages, fetch_projects},
    logger::{self, print},
};
use resolve_path::PathResolveExt;
use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::io::{BufRead, BufReader};
// use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tabled::Table;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub enum EntryPoint {
    Serve {
        #[arg(long, default_value = "5339")]
        port: u16,
    },
    Cli(Cli),
}

/// 󰉊 Blazing fast project manager CLI 󰉊
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    //   #[arg(optional = true)]
    // path: String,
}
#[derive(Subcommand, Debug, Clone)]
enum TodoActions {
    Show {
        ///Path to the matching project.
        path: Option<String>,
        //TODO add tags ?
    },
    Add {
        ///Your new todo's title
        todo_title: String,
        ///Path to the matching project.
        path: Option<String>,
        //TODO add tags ?
    },
}

#[derive(Subcommand, Debug, Clone)]
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
#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// [EXPERIMENTAL] SSH into a server. Uses dashline cli.
    // Ssh {
    //     #[arg(short, long)]
    //     new: Option<String>,
    //     name: String,
    //     user: Option<String>,
    //     host: Option<String>,
    // },
    Retrieve {
        #[arg(long)]
        host: String,
    },
    Attach,

    /// Store a key-value. Requires master password.
    Store {
        #[arg(long)]
        host: String,
    },
    /// Move to specific project
    Move {
        name: String,
        //
        // /// Name of the person to greet
        // #[arg(short, long)]
        // name: String,
        //
    },
    Todo {
        #[command(subcommand)]
        action: TodoActions,
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
        /// Shows the language or project in a full GUI rather than in the terminal.
        #[arg(short, long, default_value_t = false)]
        gui: bool,
    },
    // [EXPERIMENTAL] Run a script (associated with a project)
    // Run {
    //     //NOTE : This order is important, setting command before name allows name to become a
    //     //positional argument
    //     #[arg(index = 0)]
    //     name: Option<String>,
    //     /// If not present, defaults to current directory.
    //     #[arg(num_args(0..))]
    //     command: Option<String>,
    // },
}

pub type ActionResult = Result<(), ()>;

impl Cli {
    // TODO : Fix that mess for run & SSH. Disabled for now.
    pub fn parse_to_action(&self, state: Arc<DaemonState>, is_daemon: bool) -> ActionResult {
        // let dt_start = chrono::Utc::now();
        let timestamp_start = SystemTime::now();
        let cli = self;
        let mut default_settings = logger::TablingOptionsBuilder::default();
        let settings = default_settings
            .color(Some(String::from("default")))
            .first_row_color(Some(String::from("blue")))
            .first_col_color(Some(String::from("magenta")))
            .header(Some(String::from("Query result :")));
        let pool = state.database.clone();
        let mut conn = match pool.get() {
            Ok(conn) => conn,
            Err(err) => return Err(()),
        };

        run_migration(&mut conn);
        match &cli.command {
            Commands::Attach => {
                use std::io::Write;
                crate::server::daemon::ENABLE_TCP.with_borrow(|is_enabled| {
                    if *is_enabled {
                        crate::server::daemon::TCP_OUTPUT.with_borrow_mut(|out| {
                            loop {
                                // let e = async {
                                //
                                //  };
                                // Future::poll
                                let buffer = crate::server::daemon::MESSAGES.clone();
                                let mut buf = buffer.inner.lock().unwrap();
                                while buf.is_empty() {
                                    buf = buffer.wait(buf).unwrap();
                                }
                                // TODO: For now this only works for a single attached entity,
                                // since we drain the buffer.
                                while let Some(line) = buf.pop_front() {
                                    if let Some(writer) = out {
                                        // let mut reader =
                                        //     BufReader::new(writer.try_clone().unwrap());
                                        // let mut line_read = String::new();
                                        // reader.read_line(&mut line_read);
                                        let _ = writeln!(writer, "{}", line);
                                    };
                                }
                            }
                        });
                    }
                });
            }
            Commands::Retrieve { host } => {
                let key = auth::manager::requires_password(&mut conn);
                let cleartext = auth::manager::retrieve_encrypted(
                    &key,
                    CryptoFilterType::Host(host.to_owned()),
                    &mut conn,
                );
                auth::manager::show_password(&cleartext).unwrap();
            }
            Commands::Store { host } => {
                // tcp_tcp_println!("our host : {host}");
                let key = auth::manager::requires_password(&mut conn);
                let data = auth::manager::hidden_user_input(0);
                auth::manager::store_encrypted(&data, host, &key, &mut conn);
            }
            // Commands::Ssh {
            //     new,
            //     name,
            //     user,
            //     host,
            //     // TODO : Generate the struct here. Messing up the argument order is too easy
            //
            // } => my_cli::ssh::ssh_into(&mut conn, new, name, host, user, settings),
            // Commands::Move { name } => mover::move_to(&mut conn, name),
            // Commands::Run { name, command } => run_command(&mut conn, name, command),
            Commands::Add { add_type } => match &add_type {
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
                    let abs = match std::path::absolute(&path_item) {
                        Ok(absolute_path) => absolute_path,
                        Err(err) => panic!("An error occured while processing the path : {}", err),
                    };

                    match abs.to_str() {
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
                // TODO : add gui
                TypeOfAdds::Todo { todo_title, path } => {
                    if is_daemon {
                        //TODO call self
                        let args = self.command.to_args();
                        spawn(args);
                        return Ok(());
                    };
                    open_todo(path, Some(todo_title), &state);
                }
            },

            Commands::Move { name } => move_to(name, &mut conn),
            //TODO add gui
            Commands::Todo { action } => match action {
                TodoActions::Show { path } => {
                    if is_daemon {
                        //TODO call self
                        let args = self.command.to_args();
                        spawn(args);
                        return Ok(());
                    };

                    open_todo(path, None, &state);
                }
                TodoActions::Add { todo_title, path } => {
                    if is_daemon {
                        //TODO call self
                        let args = self.command.to_args();
                        spawn(args);
                        return Ok(());
                    };

                    open_todo(path, Some(todo_title), &state);
                }
            },

            //ADD GUI
            Commands::Show {
                searchterm,
                lang_query,
                gui,
            } => {
                if *gui {
                    if is_daemon {
                        //TODO call self
                        let args = self.command.to_args();
                        spawn(args);
                        return Ok(());
                    };
                    let options = eframe::NativeOptions {
                        viewport: egui::ViewportBuilder::default().with_maximized(true), //.with_inner_size([320.0, 240.0])
                        renderer: eframe::Renderer::Glow,
                        ..Default::default()
                    };
                    let p = single_project(&mut conn, searchterm);
                    let project: ProjectEditor = (&mut conn, p).into();
                    let viewer = ProjectViewer(project);
                    eframe::run_native(
                        "Project Overview",
                        options,
                        Box::new(|cc| {
                            FONTS::add_rounded_icons(&cc.egui_ctx);
                            // This gives us image support:
                            // egui_extras::install_image_loaders(&cc.egui_ctx);

                            Ok(Box::new(viewer))
                        }),
                    )
                    .unwrap();
                    // return Ok(());
                }
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
            _ => tcp_println!("Unsupported command..."),
        }

        //  let dt_end = chrono::Utc::now();
        let timestamp_end = SystemTime::now();
        let duration = timestamp_end.duration_since(timestamp_start).unwrap();
        match &cli.command {
            Commands::Move { name: _ } => {
                trace!("Total query time : {} ms.", (duration.as_millis()))
            }
            _ => tcp_println!("Total query time : {} ms.", (duration.as_millis())),
        };
        Ok(())
    }
}

fn open_todo(
    path: &Option<String>,
    todo_title: Option<&str>,
    state: &DaemonState,
    // warnings: &Mutex<Vec<Warning>>,
) {
    let warnings = &state.warnings;
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
                tcp_println!("Err ----------- \n {}", err);
                panic!("An error occured. The path provided is invalid.");
            }
        },
        None => current.canonicalize().unwrap(),
    };
    let mut conn = match state.database.get() {
        Ok(conn) => conn,
        // TODO HANDLE ERROR
        Err(e) => return,
    };
    let mut projects = all_projects(&mut conn);
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

                // TODO remove unwrap
                warnings.lock().unwrap().push(Warning::new(&msg, true))
            }
        };
    }
    if let Some(project) = target_proj {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_maximized(true), //.with_inner_size([320.0, 240.0])
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        };

        let mut list: TodoList = TodoList::default_state(state.database.clone());
        list.with_parent(project);
        list.retrieve();
        if let Some(title) = todo_title {
            let todo = TodoEditor::new("md", title, "", "", StoredId::New(0), project.id);
            list.add(todo);
        }

        eframe::run_native(
            "Your Todos",
            options,
            Box::new(|cc| {
                FONTS::add_rounded_icons(&cc.egui_ctx);
                // This gives us image support:
                // egui_extras::install_image_loaders(&cc.egui_ctx);

                Ok(Box::new(list))
            }),
        )
        .unwrap();
    }
}
pub trait ToArgs {
    fn to_args(&self) -> Vec<OsString>;
}

impl ToArgs for Cli {
    fn to_args(&self) -> Vec<OsString> {
        self.command.to_args()
    }
}
impl ToArgs for TodoActions {
    fn to_args(&self) -> Vec<OsString> {
        match self {
            TodoActions::Show { path } => {
                let mut v = vec![OsString::from("show")];
                if let Some(p) = path {
                    v.push(OsString::from(p));
                };
                v
            }
            TodoActions::Add { todo_title, path } => {
                let mut v = vec![OsString::from("add"), OsString::from(todo_title)];
                if let Some(p) = path {
                    v.push(OsString::from(p));
                };
                v
            }
        }
    }
}
impl ToArgs for TypeOfAdds {
    fn to_args(&self) -> Vec<OsString> {
        match self {
            TypeOfAdds::Lang { language_name } => {
                vec![OsString::from("lang"), OsString::from(language_name)]
            }
            TypeOfAdds::Project {
                project_path,
                project_name,
                language,
            } => vec![
                OsString::from("project"),
                OsString::from(project_path),
                OsString::from(project_name),
                OsString::from("-l"),
                OsString::from(language),
            ],
            TypeOfAdds::Todo { todo_title, path } => {
                let mut v = vec![OsString::from("todo"), OsString::from(todo_title)];
                if let Some(p) = path {
                    v.push(OsString::from(p));
                };
                v
            }
        }
    }
}
impl ToArgs for Commands {
    fn to_args(&self) -> Vec<OsString> {
        match self {
            Commands::Retrieve { host } => vec![
                OsString::from("retrieve"),
                OsString::from("--host"),
                OsString::from(host),
            ],

            Commands::Store { host } => vec![
                OsString::from("store"),
                OsString::from("--host"),
                OsString::from(host),
            ],
            Commands::Move { name } => vec![OsString::from("move"), OsString::from(name)],
            Commands::Todo { action } => {
                let mut v = action.to_args();
                v.insert(0, OsString::from("todo"));
                v
            }
            Commands::Add { add_type } => {
                let mut v = add_type.to_args();
                v.insert(0, OsString::from("add"));
                v
            }
            Commands::Attach => {
                vec![OsString::from("attach")]
            }
            Commands::Show {
                searchterm,
                lang_query,
                gui,
            } => {
                let mut v = vec![OsString::from("show")];
                if *lang_query {
                    v.push(OsString::from("-lang_query"));
                };
                if *gui {
                    v.push(OsString::from("--gui"));
                }
                v.push(OsString::from(searchterm));
                v
            } // Commands::Run { name, command } => {
              //     let mut v = vec![OsString::from("show")];
              //     if let Some(n) = name {
              //         v.push(OsString::from(n));
              //     };
              //     if let Some(c) = command {
              //         v.push(OsString::from(c));
              //     }
              //     v
              // }
        }
    }
}

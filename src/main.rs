use clap::{Parser, Subcommand};
use my_cli::{
    database::{
        create_language, create_project, establish_connection, fetch_languages, fetch_projects,
    },
    mover,
};
use resolve_path::PathResolveExt;
/// Simple program to greet a person
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
    // (add) Language
    L {
        language_name: String,
    },
    // (add) Project
    P {
        project_path: String,
        project_name: String,

        //The language to register the project with
        #[arg(short, long)]
        language: String,
    },
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Move to specific project
    Move {
        name: Option<String>,
        //
        // /// Name of the person to greet
        // #[arg(short, long)]
        // name: String,
        //
    },
    // Add languages and projects
    Add {
        #[command(subcommand)]
        add_type: TypeOfAdds,
        // path: String,
        // name: String,
        // #[arg(short, long)]
        // language: Option<String>,
    },
    Show {
        language: String,
        //Indicate whether you wanna show languages or projects.
        #[arg(short, long, default_value_t = false)]
        lang_query: bool,
    },
}
fn parse() {
    let cli = Cli::parse();

    let mut conn = establish_connection();
    match &cli.command {
        Commands::Move { name } => mover::move_to(&mut conn, name),
        Commands::Add { add_type } => match &add_type {
            TypeOfAdds::L { language_name } => {
                let lg = create_language(&mut conn, language_name);
                println!("{:?}", lg);
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
                        println!("{:?}", prj);
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
                println!("{:?}", fetch_languages(&mut conn, language));
            } else {
                println!("{:?}", fetch_projects(&mut conn, language));
            }
        }
    }
}

fn main() {
    //  let _ = parse();
    parse();
    // Ok(());
}

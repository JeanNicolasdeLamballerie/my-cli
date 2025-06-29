use clap::Parser;
use log::trace;
use my_cli::database::generate_pool;
use my_cli::exceptions::HandleException;
use my_cli::logger::setup_logger;
use my_cli::server::daemon::{start_daemon, DaemonState};
use my_cli::tcp_println;

use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
pub const NL: &str = "\r\n";

#[cfg(not(target_os = "windows"))]
pub const NL: &str = "\n";
fn main() {
    trace!("Entry point.");
    let _ = setup_logger();
    trace!("Generating warnings and state.");
    // let warnings: Arc<Mutex<Vec<Warning>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_state = Arc::new(DaemonState {
        database: generate_pool(),
        warnings: Mutex::new(Vec::new()),
    });
    let entry = my_cli::cli::EntryPoint::parse();
    match entry {
        my_cli::cli::EntryPoint::Serve { port } => {
            tcp_println!("Serve endpoint reached.");
            start_daemon(port, shared_state.clone()).unwrap()
        }
        //TODO remove unwrap
        my_cli::cli::EntryPoint::Cli(cli) => {
            cli.parse_to_action(shared_state.clone(), false).unwrap()
        }
    }

    // view();
    trace!("Parsing...");
    // TODO add environment variable port / Err handling
    trace!("Done parsing. moving on to warnings.");
    let warnings = &shared_state.warnings;
    //Todo avoid unwrap ?
    for warning in warnings.lock().unwrap().iter() {
        warning.warn();
    }
    trace!("Exiting.");
}

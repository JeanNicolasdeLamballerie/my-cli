use clap::Parser;
use log::trace;
use my_cli::database::generate_pool;
use my_cli::exceptions::HandleException;
use my_cli::logger::{setup_logger, AsynchronousStatus, ASYNC_STATUS};
use my_cli::server::daemon::{start_daemon, DaemonState};
use my_cli::server::log_queue::{LogQueue, LOG_QUEUE};

use std::sync::{Arc, Mutex};

#[cfg(target_os = "windows")]
pub const NL: &str = "\r\n";

#[cfg(not(target_os = "windows"))]
pub const NL: &str = "\n";

fn main() {
    let entry = my_cli::cli::EntryPoint::parse();
    // let warnings: Arc<Mutex<Vec<Warning>>> = Arc::new(Mutex::new(Vec::new()));
    let shared_state = Arc::new(DaemonState {
        database: generate_pool(),
        warnings: Mutex::new(Vec::new()),
    });
    println!("pre-match : {:?}", ASYNC_STATUS.get());
    match entry {
        my_cli::cli::EntryPoint::Serve { port } => {
            ASYNC_STATUS
                .set(AsynchronousStatus::Available)
                .expect("Failed to set TCP status as Available");
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime");

            rt.block_on(async {
                LOG_QUEUE
                    .set(Arc::new(LogQueue::new()))
                    .expect("Failed to set the log queue at run time...");
                let _ = setup_logger(AsynchronousStatus::Available);
                start_daemon(port, shared_state.clone()).await.unwrap();
            });
        }

        my_cli::cli::EntryPoint::Cli(cli) => {
            ASYNC_STATUS
                .set(AsynchronousStatus::Unavailable)
                .expect("Failed to set TCP status as Unavailable");
            let _ = setup_logger(AsynchronousStatus::Unavailable);
            cli.parse_to_action(shared_state.clone()).unwrap();
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

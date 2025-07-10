use once_cell::sync::OnceCell;
use std::collections::VecDeque;
use std::env::current_exe;
use std::io::{BufRead, Read, Write};
use std::ops::Deref;
// use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task_local;

use clap::{CommandFactory, Parser};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use log::{debug, error, info, trace, warn};

use crate::exceptions::Warning;
use tokio::io::AsyncWriteExt;
// Your Clap-based CLI parser
use crate::cli::Cli;
use crate::server::log_queue::LOG_QUEUE;
use crate::server::{port, shim_gen}; // assuming this is your Clap parser
                                     // use lazy_static::lazy_static;
                                     // use tokio::task::LocalKey

// lazy_static! {
//     pub static ref MESSAGES: Arc<Q> = Arc::new(Queue::default());
// }

// thread_local! {
//     pub static TCP_OUTPUT: Mutex<Option<Arc<Mutex<dyn Write + Send>>>> = Mutex::new(None);
// }
//
// Custom tcp_println! that checks if we're inside a TCP session
// #[macro_export]
// macro_rules! tcp_println {
//     // ($($arg:tt)*) => {{
//     //     use std::io::Write;
//     //     use std::sync::Arc;
//     //     use $crate::server::daemon::TcpStatus;
//     //     let line = format!($($arg)*);
//     //
//     //     use tokio::io::AsyncWriteExt;
//     //     match $crate::server::daemon::BROADCAST.get().unwrap() {
//     //     TcpStatus::Available => {
//     //
//     //         $crate::server::daemon::TCP_OUTPUT.with(|writer| {
//     //             let writer = writer.clone();
//     //             tokio::spawn(async move {
//     //                 let mut w = writer.lock().await;
//     //                 let _ = w.write_all(line.as_bytes()).await;
//     //                 let _ = w.write_all(b"\n").await;
//     //             });
//     //         });
//     //     }
//     //     TcpStatus::Unavailable => {
//     //
//     //     }
//     //     }
//     //
//     //
//     //    let msgs = $crate::server::daemon::MESSAGES.clone();
//     //     msgs.add_and_notify(line);
//     // }};
// }
#[macro_export]
macro_rules! tcp_log {
    // Async: stream + custom level
    ($stream:expr => $level:ident, $($arg:tt)+) => {{
        use tokio::io::AsyncWriteExt;
        let msg = format!($($arg)+);
        log::log!(log::Level::$level, "{}", msg);

        let mut stream = $stream;
        async move {
            let _ = stream.write_all(msg.as_bytes()).await;
            let _ = stream.write_all(b"\n").await;
        }
    }};

    // Async: stream only, default level
    ($stream:expr => $($arg:tt)+) => {{
        use tokio::io::AsyncWriteExt;
        let msg = format!($($arg)+);
        log::info!("{}", msg);

        let stream = $stream;
        async move {
            let _ = stream.write_all(msg.as_bytes()).await;
            let _ = stream.write_all(b"\n").await;
        }
    }};

    // Sync: custom log level
    ($level:ident, $($arg:tt)+) => {
        log::log!(log::Level::$level, $($arg)+);
    };

    // Sync: default info level
    ($($arg:tt)+) => {
        log::info!($($arg)+);
    };
}

// #[macro_export]
// macro_rules! tcp_log {
//
// ($stream:expr => $($arg:tt)+) => {{
//         use tokio::io::AsyncWriteExt;
//         let msg = format!($($arg)+);
//         log::info!("{}", msg);
//
//         let stream = $stream;
//     async move {
//             let _ = stream.write_all(msg.as_bytes()).await;
//             let _ = stream.write_all(b"\n").await;
//         }
//     }};
//   ($stream:expr => $level:ident, $($arg:tt)+) => {{
//         use tokio::io::AsyncWriteExt;
//         let msg = format!($($arg)+);
//         log::log!($level, format!("{}", msg));
//
//         let mut stream = $stream.clone();
//         async move {
//             let _ = stream.write_all(msg.as_bytes()).await;
//             let _ = stream.write_all(b"\n").await;
//         }
//     }};
//     // Default log level
//     ($($arg:tt)*) => {
//         log::log!(log::Level::Info, $($arg)*);
//     };
//
//     // Custom log level
//     ($level:ident, $($arg:tt)*) => {
//         log::log!($level, $($arg)*);
//     };
// }
//
pub struct DaemonState {
    pub database: Pool<ConnectionManager<SqliteConnection>>,
    pub warnings: std::sync::Mutex<Vec<Warning>>,
}
/// Starts the long-lived TCP server listening for CLI requests.
pub async fn start_daemon(port: u16, shared_state: Arc<DaemonState>) -> tokio::io::Result<()> {
    // Mutex
    tcp_log!("Starting asynchronous daemon...");
    port::write_port(&port.to_string()).expect("Could not write to the temp file... shutting down");
    tcp_log!("Generating shims... Do not exit the process...");
    shim_gen::generate_shims(&current_exe()?).unwrap();
    let mut cmd = Cli::command();

    tcp_log!("Generating autocompletion... This is a one time process, do not exit the process...");
    shim_gen::generate_completions(&mut cmd).unwrap();
    shim_gen::print_completion_instructions();
    // }
    let listener = TcpListener::bind(("127.0.0.1", port)).await?;
    info!("CLI daemon listening on port {}", port);
    tcp_log!("Daemon is now running.");

    while let Ok(stream) = listener.accept().await {
        let stream = stream.0;
        let state = shared_state.clone();
        tokio::spawn(async move {
            if let Err(err) = handle_client(stream, state).await {
                warn!("Client handler error: {}", err);
            };
        });
    }

    Ok(())
}
async fn handle_client(mut stream: TcpStream, state: Arc<DaemonState>) -> tokio::io::Result<()> {
    let peer = stream.peer_addr()?;
    // let (reader,mut  writer) = stream.into_split();
    debug!("Incoming connection from {}", peer);

    let mut reader = tokio::io::BufReader::new(&mut stream).lines();
    // let mut line = String::new();
    if let Ok(Some(line)) = reader.next_line().await {
        // Expect one command line per connection (you can extend this later)
        // if line == 0 {
        //     warn!("Empty connection from {}", peer);
        //     return Ok(());
        // }

        trace!("Received raw command line: {:?}", line);
        let mut args = shell_words::split(line.trim_end()).unwrap_or_else(|err| {
            warn!("Failed to parse args: {}", err);
            vec![]
        });

        if args.is_empty() {
            stream.write_all("Invalid command".as_bytes()).await?;
            // writeln!(writer, "Invalid command")?;
            return Ok(());
        }
        args.insert(0, String::from("rush"));

        debug!("Parsed arguments: {:?}", args);

        // Dispatch the command using Clap
        match Cli::try_parse_from(args) {
            Ok(cli) => {
                trace!("Dispatching CLI command: {:?}", cli);
                if let Err(e) = cli.parse_to_action(state, &mut stream).await {
                    //TODO : adjust state errors !
                    stream.write_all("Error...".as_bytes()).await?;
                    error!("Command failed...");
                }
            }
            Err(e) => {
                stream
                    .write_all(format!("Error.. {}.", e).as_bytes())
                    .await?;
                // warn!("Failed to parse CLI input: {}", e);
            }
        };
    };
    Ok(())
}

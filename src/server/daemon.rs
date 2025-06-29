use std::collections::VecDeque;
use std::env::current_exe;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Condvar, LockResult, Mutex, MutexGuard};
use std::thread;

use clap::{CommandFactory, Parser};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::SqliteConnection;
use log::{debug, error, info, trace, warn};

use crate::exceptions::Warning;
// Your Clap-based CLI parser
use crate::cli::Cli;
use crate::server::{port, shim_gen}; // assuming this is your Clap parser
use lazy_static::lazy_static;
use std::cell::RefCell;
// use std::io::Write;
#[derive(Default, Debug)]
pub struct Queue {
    pub inner: Mutex<VecDeque<String>>,
    notify: Condvar,
}
impl Queue {
    // type InnerLock: MutexGuard<Mutex<VecDeque<String>>>;
    pub fn add_and_notify(&self, value: String) {
        let mut lock = self.inner.lock().unwrap();
        lock.push_back(value);
        drop(lock);
        self.notify.notify_all();
    }
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> LockResult<MutexGuard<'a, T>> {
        self.notify.wait(guard)
    }
}
// impl Deref
// loop {
//     let mut buf = log_buf.queue.lock().unwrap();
//     while buf.is_empty() {
//         buf = log_buf.notify.wait(buf).unwrap();
//     }
//
//     while let Some(line) = buf.pop_front() {
//         // Send line to TCP
//     }
// }
//
lazy_static! {
    pub static ref MESSAGES: Arc<Queue> = Arc::new(Queue::default());
}
thread_local! {
    pub static TCP_OUTPUT: RefCell<Option<TcpStream>> = RefCell::new(None);
    pub static ENABLE_TCP: RefCell<bool> =const{RefCell::new(false)};
    pub static CURRENT_INDEX : RefCell<usize> = const{RefCell::new(0)};
}
// thread_local! {
//     pub static TCP_OUTPUT: Mutex<Option<Arc<Mutex<dyn Write + Send>>>> = Mutex::new(None);
// }
//
// Custom tcp_println! that checks if we're inside a TCP session
#[macro_export]
macro_rules! tcp_println {
    ($($arg:tt)*) => {{
        use std::io::Write;
        let line = format!($($arg)*);
        $crate::server::daemon::ENABLE_TCP.with_borrow(|is_enabled| {
            if *is_enabled {
                $crate::server::daemon::TCP_OUTPUT.with_borrow_mut(|out| {
                    if let Some(writer) = out {
                        let _ = writeln!(writer, "{}", line);
                    };
                });
            }
        });
       let msgs = $crate::server::daemon::MESSAGES.clone();
        msgs.add_and_notify(line);
    }};
}

// Call this at the start of a TCP command handler
pub fn set_tcp_output(writer: TcpStream) {
    ENABLE_TCP.with_borrow_mut(|is_enabled| {
        *is_enabled = true;
    });
    TCP_OUTPUT.with_borrow_mut(|output| {
        *output = Some(writer);
    });
}

fn clear_tcp_output() {
    TCP_OUTPUT.with_borrow_mut(|output| {
        *output = None;
    });
}

pub struct DaemonState {
    pub database: Pool<ConnectionManager<SqliteConnection>>,
    pub warnings: Mutex<Vec<Warning>>,
}
/// Starts the long-lived TCP server listening for CLI requests.
pub fn start_daemon(port: u16, shared_state: Arc<DaemonState>) -> std::io::Result<()> {
    tcp_println!("Starting daemon...");
    port::write_port(&port.to_string()).expect("Could not write to the temp file... shutting down");
    tcp_println!("Generating shims... Do not exit the process...");
    shim_gen::generate_shims(&current_exe()?).unwrap();
    let mut cmd = Cli::command();

    tcp_println!(
        "Generating autocompletion... This is a one time process, do not exit the process..."
    );
    shim_gen::generate_completions(&mut cmd).unwrap();
    shim_gen::print_completion_instructions();
    // }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    info!("CLI daemon listening on port {}", port);
    tcp_println!("Daemon is now running.");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = shared_state.clone();
                thread::spawn(move || {
                    set_tcp_output(stream.try_clone().expect("Failed to clone TCP stream..."));
                    if let Err(err) = handle_client(stream, state) {
                        warn!("Client handler error: {}", err);
                    }
                    clear_tcp_output();
                });
            }
            Err(e) => error!("Failed to accept connection: {}", e),
        }
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, state: Arc<DaemonState>) -> std::io::Result<()> {
    let peer = stream.peer_addr()?;
    debug!("Incoming connection from {}", peer);

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();

    // Expect one command line per connection (you can extend this later)
    if reader.read_line(&mut line)? == 0 {
        warn!("Empty connection from {}", peer);
        return Ok(());
    }

    trace!("Received raw command line: {:?}", line);
    let mut args = shell_words::split(line.trim_end()).unwrap_or_else(|err| {
        warn!("Failed to parse args: {}", err);
        vec![]
    });

    if args.is_empty() {
        writeln!(stream, "Invalid command")?;
        return Ok(());
    }
    args.insert(0, String::from("rush"));

    debug!("Parsed arguments: {:?}", args);

    // Dispatch the command using Clap
    match Cli::try_parse_from(args) {
        Ok(cli) => {
            trace!("Dispatching CLI command: {:?}", cli);
            if let Err(e) = cli.parse_to_action(state, true) {
                //TODO : adjust state errors !
                writeln!(stream, "Error...")?;
                error!("Command failed...");
            }
        }
        Err(e) => {
            writeln!(stream, "{}", e)?;
            // warn!("Failed to parse CLI input: {}", e);
        }
    }

    Ok(())
}

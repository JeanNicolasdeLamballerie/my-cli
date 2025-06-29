use std::{env::current_exe, ffi::OsString, thread::spawn as spawn_thread};

use log::trace;

pub mod daemon;
pub mod port;
pub mod shim_gen;

pub fn spawn(args: Vec<OsString>) {
    spawn_thread(move || {
        let mut cmd = std::process::Command::new(current_exe().unwrap());
        trace!("Spawning main threaded gui or executing through cli options, not the TCP server.");
        cmd.arg("cli");
        for arg in args.iter() {
            cmd.arg(arg);
        }
        cmd.spawn().unwrap().wait().unwrap();
    });
}

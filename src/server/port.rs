use std::env;
use std::fs;
use std::path::PathBuf;

use crate::config::port_file_path;

const DEFAULT_PORT: &str = "5339";

pub enum TCPPort {
    Present(String),
    Missing(String),
}
pub fn read_port() -> TCPPort {
    let port_path = port_file_path();
    match fs::read_to_string(&port_path) {
        Ok(contents) => TCPPort::Present(contents.trim().to_string()),
        Err(_) => TCPPort::Missing(DEFAULT_PORT.to_string()),
    }
}

pub fn write_port(port: &str) -> std::io::Result<()> {
    let path = port_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, port)
}

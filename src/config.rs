use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct MyConfig {
    version: u8,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        Self { version: 0 }
        // _ => error_default(),
    }
}

// fn error_default() -> MyConfig {
//     etcp_println!("An error occured while generating the configuration file.");
//     return MyConfig {
//         version: 0,
//     };
// }
//
use directories::ProjectDirs;
use std::io;
use std::path::PathBuf;

/// Returns a ProjectDirs object scoped to com.example.dekharen-cli-daemon
fn project_dirs() -> io::Result<ProjectDirs> {
    ProjectDirs::from("com", "dekharen", "rush").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Could not determine project directories",
        )
    })
}

// Config directory: for user-editable files like shims or config.toml
pub fn config_dir() -> io::Result<PathBuf> {
    let path = project_dirs()?.config_dir().to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

// Data directory: for persistent files like the SQLite database
pub fn data_dir() -> io::Result<PathBuf> {
    let path = project_dirs()?.data_dir().to_path_buf();
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

// Temp directory: for ephemeral files like the TCP port
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir().join("dekharen-cli-daemon")
}

// Individual file paths

pub fn port_file_path() -> PathBuf {
    temp_dir().join("port")
}

pub fn db_path() -> io::Result<PathBuf> {
    Ok(data_dir()?.join("database.sqlite"))
}
//
// pub fn config_file_path() -> io::Result<PathBuf> {
//     Ok(config_dir()?.join("config.toml"))
// }

pub fn shims_ps_path() -> io::Result<PathBuf> {
    Ok(config_dir()?.join("shims.psm1"))
}

pub fn shims_sh_path() -> io::Result<PathBuf> {
    Ok(config_dir()?.join("shims.sh"))
}

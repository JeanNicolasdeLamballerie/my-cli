use serde_derive::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
pub struct MyConfig {
    version: u8,
}

/// `MyConfig` implements `Default`
impl ::std::default::Default for MyConfig {
    fn default() -> Self {
        return Self { version: 0 };
        // _ => error_default(),
    }
}

// fn error_default() -> MyConfig {
//     eprintln!("An error occured while generating the configuration file.");
//     return MyConfig {
//         version: 0,
//     };
// }

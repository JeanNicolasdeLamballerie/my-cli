use std::time::*;
///This trait should implement displaying errors or success.
pub trait Feedback<T> {
    fn process(&mut self, ui: &mut egui::Ui) -> ();
}

/// Struct holding the error (or success) values to display. Calling `.process`
/// will show them on the screen.
pub struct Log<T>
where
    Log<T>: Feedback<Log<T>>,
{
    value: Vec<T>,
}

// impl Log<Result<Success, DatabaseError>> {
//     pub fn new(logs: Vec<Result<Success, DatabaseError>>) -> Self {
//         Self { value: logs }
//     }
// }

impl<T> Log<T>
where
    Log<T>: Feedback<Log<T>>,
{
    pub fn new(logs: Vec<T>) -> Self {
        Self { value: logs }
    }
}

//TODO can be refactored in a more general to-display error
//

#[derive(Debug)]
pub struct DatabaseError {
    message: String,
}
impl DatabaseError {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

pub enum SuccessType {
    Database,
    File,
}

pub struct Success {
    timestamp: SystemTime,
    success_type: SuccessType,
    message: String,
}
impl Success {
    pub fn new(message: String, success_type: SuccessType) -> Self {
        Self {
            timestamp: SystemTime::now(),
            message,
            success_type,
        }
    }
}

pub struct DatabaseSuccess;

// fn abc() {
//     let e: Log<&str> = Log {
//         value: Vec::from(["a", "b", "c"]),
//     };
// }
// impl Feedback<Log<&str>> for Log<&str> {
//     fn process(&mut self, ui: &egui::Ui) -> () {}
// }

impl Feedback<Log<Result<Success, DatabaseError>>> for Log<Result<Success, DatabaseError>> {
    fn process(&mut self, ui: &mut egui::Ui) -> () {
        ui.vertical(|ui| {
            for log in &self.value {
                match log {
                    Ok(s) => {
                        let time = s.timestamp.elapsed().unwrap().as_secs();
                        //TODO add icon success / failure
                        ui.label(format!("[{} seconds ago] - {}", time, s.message));
                        ui.separator();
                    }
                    Err(db_err) => {
                        ui.label(
                            egui::RichText::new(format!("ERR - {}", db_err.message))
                                .color(egui::Color32::from_hex("#FF0000").unwrap()),
                        );
                        ui.separator();
                    }
                }
            }
        });
    }
}

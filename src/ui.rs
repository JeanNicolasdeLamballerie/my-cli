use std::time::*;

use egui::Align;

///This trait should implement displaying errors or success.
pub trait Feedback<T> {
    fn process(&mut self, ui: &mut egui::Ui) -> ();
}

/// Struct holding the error (or success) values to display. Calling `.process`
/// will show them on the screen.
pub struct Log<T>
where
    Log<T>: View,
{
    value: Vec<T>,
    should_scroll: bool,
}

// impl Log<Result<Success, DatabaseError>> {
//     pub fn new(logs: Vec<Result<Success, DatabaseError>>) -> Self {
//         Self { value: logs }
//     }
// }

impl<T> Log<T>
where
    Log<T>: View,
{
    pub fn new(logs: Vec<T>) -> Self {
        Self {
            value: logs,
            should_scroll: false,
        }
    }
    pub fn push(&mut self, element: T) {
        self.value.push(element);
    }
    pub fn should_scroll(&mut self) {
        self.should_scroll = true;
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
    pub fn success_message(self) -> String {
        match self.success_type {
            SuccessType::File => "(file operation)".into(),
            SuccessType::Database => "(database operation)".into(),
        }
    }
}
// impl Feedback<Log<Result<Success, DatabaseError>>> for Log<Result<Success, DatabaseError>> {
//     fn process(&mut self, ui: &mut egui::Ui) -> () {
//         ui.vertical(|ui| {
//             for log in &self.value {
//                 match log {
//                     Ok(s) => {
//                         let time = s.timestamp.elapsed().unwrap().as_secs();
//                         //TODO add icon success / failure
//                         ui.label(format!("[{} seconds ago] - {}", time, s.message));
//                         ui.separator();
//                     }
//                     Err(db_err) => {
//                         ui.label(
//                             egui::RichText::new(format!("ERR - {}", db_err.message))
//                                 .color(egui::Color32::from_hex("#FF0000").unwrap()),
//                         );
//                         ui.separator();
//                     }
//                 }
//             }
//         });
//     }
// }

pub trait View {
    ///Transforms an element into a displayed menu or view in the egui context.
    fn ui(&mut self, ui: &mut egui::Ui);
}

/// Something to view
pub trait WindowUI {
    // `&'static` so we can also use it as a key to store open/close state.
    fn name(&self) -> &str;

    /// Show windows, etc
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);
}

impl View for Log<Result<Success, DatabaseError>> {
    fn ui(&mut self, ui: &mut egui::Ui) -> () {
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
            if self.should_scroll {
                self.should_scroll = false;
                ui.scroll_to_cursor(Some(Align::TOP));
            };
        });
    }
}

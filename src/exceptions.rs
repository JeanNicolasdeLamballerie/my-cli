use std::io;
pub struct Warning {
    message: String,
    confirmation: bool,
    // action: dyn Fn() -> (),
}

pub trait Action<T> {
    fn action(&self);
}

pub trait HandleException {
    fn warn(&self);
}

impl Action<Warning> for Warning {
    fn action(&self) {
        //TODO
        println!("Do some type specific db operation or other here...");
    }
}

impl HandleException for Warning {
    fn warn(&self) {
        println!("Warning :");
        println!("Message : {}", self.message);
        let mut proceed = true;
        if self.confirmation {
            proceed = false;
            let input = get_input("Confirm action (Y/N) :");
            if input.to_lowercase() == "y" {
                proceed = true;
            }
        }
        // match self.action {
        // Some(func) => {
        if proceed {
            self.action();
        }
        // }
        // None => {}
        // }
    }
}

fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    input.trim().to_string()
}

impl Warning {
    pub fn new(message: &str, needs_confirmation: bool) -> Self {
        Self {
            message: message.into(),
            confirmation: needs_confirmation,
        }
    }
}

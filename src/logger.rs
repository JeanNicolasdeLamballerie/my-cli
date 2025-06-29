use tabled::{
    builder::Builder,
    settings::{
        object::{Columns, Rows},
        style::BorderColor,
        themes::Colorization,
        Alignment, Color, Margin, Modify, Padding, Style,
    },
    Table,
};
// enum Colors {
//     Monochrome { color:  Color::BG_BLACK | Color::FG_WHITE},
//
// }

use derive_builder::Builder as BuildStruct;

#[derive(BuildStruct, Debug)]
pub struct TablingOptions {
    color: Option<String>,
    first_row_color: Option<String>,
    first_col_color: Option<String>,
    header: Option<String>,
}
// trait Colored {
//     fn get_colors(&self) -> Option<Color>;
// }
// // todo https://docs.rs/derive_builder/latest/derive_builder/
// //todo https://vidify.org/blog/rust-parameters/
// impl TablingOptions {
//     fn new() -> Self {
//         TablingOptions {
//             color: Some("default"),
//             header: None,
//         }
//     }
// }
//
impl TablingOptions {
    pub fn get_color(&self, color: &Option<String>) -> Color {
        let blue = Color::BG_BLUE | Color::FG_BLACK;
        let green = Color::BG_GREEN | Color::FG_BLACK;
        let magenta = Color::BG_MAGENTA | Color::FG_BLACK;

        let default_colors = Color::BG_BLACK | Color::FG_WHITE;
        match &color {
            Some(color) => match color.as_str() {
                "green" => green,
                "magenta" => magenta,
                "blue" => blue,
                _ => default_colors,
            },
            None => default_colors,
        }
    }

    fn colors(&self) -> Vec<Color> {
        // Vec<Option<Color>> =
        [&self.color, &self.first_row_color, &self.first_col_color]
            .iter()
            .map(|color| self.get_color(color))
            .collect()
    }
}
pub fn print(table: &mut Table, opts: &mut TablingOptionsBuilder) {
    let e: TablingOptions = opts.build().unwrap();
    let colors = e.colors();
    let mut pre_tables: Vec<Table> = vec![];
    let default_color = [colors.first().unwrap().to_owned()];
    let first_row_color = [colors.get(1).unwrap().to_owned()];
    let first_col_color = [colors.last().unwrap().to_owned()];

    if let Some(header) = &e.header {
        let mut header_build = Builder::default();
        header_build.set_header([header]);
        let mut header_table = header_build.build();

        header_table
            .with(Style::modern())
            .with(
                Modify::new(Rows::first())
                    .with(BorderColor::filled(Color::BG_BLACK | Color::FG_BLUE)),
            )
            //     .with(Border::new().corner_bottom_right("/"))
            .with(Colorization::rows(default_color.clone()));

        pre_tables.push(header_table)
    };

    table
        .with(Style::empty())
        .with(Modify::new(Rows::new(1..)).with(Alignment::left()))
        .with(Padding::new(3, 3, 1, 1).colorize(
            Color::FG_BLUE,
            Color::FG_BLUE,
            Color::FG_BLUE,
            Color::FG_BLUE,
        ))
        //.with(Colorization::)
        .with(Colorization::rows(default_color))
        .with(Margin::new(0, 0, 0, 1))
        // .colorize(
        //     Color::BG_BLACK,
        //     Color::BG_BLACK,
        //     Color::BG_BLACK,
        //     Color::BG_BLACK,
        // ))
        .with(Colorization::exact(first_col_color, Columns::first()))
        .with(Colorization::exact(first_row_color, Rows::first()));

    for ele in pre_tables {
        tcp_println!("{}", ele)
    }
    tcp_println!("{}", table);
    ///////////////////////////////////////////////
    // let color1 = Color::BG_BLACK | Color::FG_WHITE;
    // // let color2 = Color::BG_GREEN | Color::FG_BLACK;
    // let color3 = Color::BG_MAGENTA | Color::FG_BLACK;
    // let color4 = Color::BG_BLUE | Color::FG_BLACK;

    //   let _ =
}

use std::{path::PathBuf, time::SystemTime};

use crate::{config, tcp_println};
pub fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_nanos(SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        // .chain(std::io::stdout())
        .chain(fern::log_file(get_log_path())?)
        .chain(fern::Output::call({
            let buf = crate::server::daemon::MESSAGES.clone();
            move |record| {
                buf.add_and_notify(record.args().to_string());
            }
        }))
        .apply()?;
    Ok(())
}
fn get_log_path() -> PathBuf {
    let local = &config::data_dir().expect("Could not determine database path !");
    match local.try_exists() {
        Ok(exists) => {
            if !exists {
                std::fs::create_dir_all(local).unwrap();
            }
        }
        Err(err) => panic!(
            "An error occured while acquiring the local directories : {}",
            err
        ),
    }
    local.join("cli.log")
}

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     setup_logger()?;
//
//     info!("Hello, world!");
//     warn!("Warning!");
//     debug!("Now exiting.");
//
//     Ok(())
// }
//

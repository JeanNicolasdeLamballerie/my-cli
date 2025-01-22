//

use crate::{
    database::{self},
    models::{FormattedTodo, NewTodo, UpdateTodo},
    todos::TodoId,
    ui::{DatabaseError, Success},
};

pub trait Modified {
    ///Returns whether the struct's fields are different from the backed up fields.
    fn is_modified(&self) -> bool;
    ///Resets the fields to their backed up values.
    fn reset(&mut self);
    ///Replaces the backed up value with the current values. Should be used when saving the value.
    fn replace(&mut self);
}

impl Modified for TodoEditor {
    fn is_modified(&self) -> bool {
        self.code != self.back_up
            || self.title != self.title_backup
            || self.subtitle != self.subtitle_backup
    }
    fn reset(&mut self) {
        self.subtitle = self.subtitle_backup.clone();
        self.title = self.title_backup.clone();
        self.code = self.back_up.clone();
    }
    fn replace(&mut self) {
        self.subtitle_backup = self.subtitle.clone();
        self.title_backup = self.title.clone();
        self.back_up = self.code.clone();
    }
}

#[derive(Clone, Debug)]
pub struct TodoEditor {
    pub id: TodoId,
    pub project_id: i32,
    pub gid: String,
    pub language: String,
    pub code: String,
    back_up: String,
    pub name: String,
    pub title: String,
    title_backup: String,
    pub subtitle: String,
    subtitle_backup: String,
    pub modified: bool,
}
impl Default for TodoEditor {
    fn default() -> Self {
        let code = "// A very simple \n\n example
*fn main()* {\n\
\tprintln!(\"***Hello world!***\");\r\n\
}\n\
";
        let title = "Default title";
        let id = TodoId::Stored(0);
        let (name, gid) = extract_name_gid(&id, title);
        Self {
            id,
            gid,
            project_id: 0,
            language: "md".into(),
            code: code.into(),
            back_up: code.into(),
            name,
            title: title.into(),
            title_backup: title.into(),
            subtitle: "Default Subtitle".into(),
            subtitle_backup: "Default Subtitle".into(),
            modified: true,
        }
    }
}
/// Formats a unique string (based on Id that shouldn't change... except when saving, but we can
/// not change the gid... maybe) and a title for the window
fn extract_name_gid(id: &TodoId, title: &str) -> (String, String) {
    match id {
        TodoId::New(number_id) => (
            format!("New [{}] - {}", number_id, title),
            format!("NEW-[{}]", number_id),
        ),
        TodoId::Stored(id) => (format!("[{}] - {}", id, title), format!("[{}]", id)),
    }
}
impl TodoEditor {
    pub fn update_name(&mut self) {
        let (name, _) = extract_name_gid(&self.id, &self.title);
        self.name = name;
    }
    pub fn id_default(id: i32) -> Self {
        let mut td = TodoEditor {
            id: TodoId::Stored(id),
            ..Default::default()
        };

        let (name, gid) = extract_name_gid(&td.id, &td.title);
        td.gid = gid;
        td.name = name;
        td
    }
    pub fn new(
        language: &str,
        title: &str,
        subtitle: &str,
        code: &str,
        id: TodoId,
        project_id: i32,
    ) -> Self {
        let (name, gid) = extract_name_gid(&id, title);
        Self {
            project_id,
            id,
            gid,
            language: language.into(),
            code: code.into(),
            back_up: code.into(),
            name,
            title: title.into(),
            title_backup: title.into(),
            subtitle: subtitle.into(),
            subtitle_backup: subtitle.into(),
            modified: false,
        }
    }
    pub fn title_str(&self) -> String {
        format!("# {}\n\n## {}", self.title, self.subtitle)
    }
    pub fn to_display(&self) -> String {
        format!("{}\n\n{}", self.title_str(), self.code)
    }
}
impl crate::database::Save<FormattedTodo> for TodoEditor {
    fn save_to_db(&mut self) -> Result<Success, DatabaseError> {
        let todo = self.to_saved_format();
        let result = if todo.new {
            database::create_todo(NewTodo {
                project_id: &todo.project_id,
                title: &todo.title,
                subtitle: Some(&todo.subtitle),
                content: Some(&todo.content),
            });
            Success::new(
                format!("Successfully created todo {}.", todo.title),
                crate::ui::SuccessType::Database,
            )
        } else {
            database::update_todo(UpdateTodo {
                id: &todo.id,
                project_id: &todo.project_id,
                title: &todo.title,
                subtitle: Some(&todo.subtitle),
                content: Some(&todo.content),
            });
            Success::new(
                format!(
                    "Updated todo ({}, id {}) successfully.",
                    todo.title, todo.id
                ),
                crate::ui::SuccessType::Database,
            )
        };

        Ok(result)
    }

    fn to_saved_format(&mut self) -> FormattedTodo {
        self.into()
    }
}
impl crate::ui::WindowUI for TodoEditor {
    fn name(&self) -> &str {
        &self.name
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use crate::ui::View as _;
        egui::Window::new(self.name())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}
impl crate::ui::View for TodoEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            language,
            code,
            name: _,
            back_up: _,
            title,
            id: _,
            gid: _,
            project_id: _,
            title_backup: _,
            subtitle,
            subtitle_backup: _,
            modified: _,
        } = self;
        ui.horizontal(|ui| {
            ui.set_height(0.0);
            ui.label("Your todo...");
        });

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Title");
            ui.text_edit_singleline(title);
            ui.label("Subtitle");
            ui.text_edit_singleline(subtitle);
            // ui.label("Compile the demo with the ");
            // ui.code("syntax_highlighting");
            // ui.label(" feature to enable more accurate syntax highlighting using ");
            // ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
            // ui.label(".");
        });
        // }

        let mut theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                string,
                language,
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .desired_rows(13)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });

        self.update_name();
    }
}

impl From<crate::models::Todo> for TodoEditor {
    fn from(value: crate::models::Todo) -> Self {
        Self::new(
            "md",
            &value.title,
            &value.subtitle.unwrap_or("".into()),
            &value.content.unwrap_or("".into()),
            TodoId::Stored(value.id),
            value.project_id,
        )
    }
}
impl From<FormattedTodo> for TodoEditor {
    fn from(value: FormattedTodo) -> Self {
        let id = if value.new {
            TodoId::New(value.id)
        } else {
            TodoId::Stored(value.id)
        };
        Self::new(
            "md",
            &value.title,
            &value.subtitle,
            &value.content,
            id,
            value.project_id,
        )
    }
}
// impl Into<TodoEditor> for FormattedTodo {
//     fn into(self) -> TodoEditor {
//         let id = if self.new {
//             TodoId::New(self.id)
//         } else {
//             TodoId::Stored(self.id)
//         };
//         TodoEditor::new(
//             "md",
//             &self.title,
//             &self.subtitle,
//             &self.content,
//             id,
//             self.project_id,
//         )
//     }
// }

use crate::{
    database::{self, establish_connection, get_language, Save},
    editor::Modified,
    models::{FormattedProject, Language, Project},
    todos::StoredId,
    ui::{DatabaseError, Success},
};
use std::path::{Path, PathBuf};

impl Modified for ProjectEditor {
    fn is_modified(&self) -> bool {
        self.project_name != self.back_up || self.path != self.path_backup
    }
    fn reset(&mut self) {
        self.path = self.path_backup.clone();
        self.project_name = self.back_up.clone();
    }
    fn replace(&mut self) {
        self.path_backup = self.path.clone();
        self.back_up = self.project_name.clone();
    }
}
// pub struct Project {
//     pub id: i32,
//     pub name: String,
//     pub path: String,
//     pub language_id: i32,
// }
#[derive(Clone, Debug)]
pub struct ProjectEditor {
    pub id: StoredId,
    pub gid: String,
    pub project_name: String,
    back_up: String,
    pub name: String,
    pub path: PathBuf,
    path_backup: PathBuf,
    pub language: Language,
    pub modified: bool,
}
impl Default for ProjectEditor {
    fn default() -> Self {
        let project_name = "Default project name";
        let path = PathBuf::from("/some/path");
        let id = StoredId::Stored(0);
        let language = Language {
            id: 0,
            name: "None".into(),
        };
        let (name, gid) = extract_name_gid(&id, project_name);
        Self {
            id,
            gid,
            language,
            project_name: project_name.into(),
            back_up: project_name.into(),
            name,
            path_backup: path.clone(),
            path,
            modified: true,
        }
    }
}
/// Formats a unique string (based on Id that shouldn't change... except when saving, but we can
/// not change the gid... maybe) and a title for the window
fn extract_name_gid(id: &StoredId, title: &str) -> (String, String) {
    match id {
        StoredId::New(number_id) => (
            format!("New [{}] - {}", number_id, title),
            format!("NEW-[{}]", number_id),
        ),
        StoredId::Stored(id) => (format!("[{}] - {}", id, title), format!("[{}]", id)),
    }
}
impl ProjectEditor {
    pub fn update_name(&mut self) {
        let (name, _) = extract_name_gid(&self.id, &self.project_name);
        self.name = name;
    }
    pub fn id_default(id: i32) -> Self {
        let mut td = ProjectEditor {
            id: StoredId::Stored(id),
            ..Default::default()
        };

        let (name, gid) = extract_name_gid(&td.id, &td.project_name);
        td.gid = gid;
        td.name = name;
        td
    }
    pub fn new(path: &Path, project_name: &str, language: &Language, id: StoredId) -> Self {
        let (name, gid) = extract_name_gid(&id, project_name);
        Self {
            id,
            gid,
            language: language.to_owned(),
            project_name: project_name.into(),
            back_up: project_name.into(),
            name,
            path: path.into(),
            path_backup: path.into(),
            modified: false,
        }
    }
}
impl Save<FormattedProject> for ProjectEditor {
    fn to_saved_format(&mut self) -> FormattedProject {
        self.into()
    }
    fn save_to_db(&mut self) -> Result<Success, DatabaseError> {
        use Save as _;
        let project = self.to_saved_format();
        let result = if project.new {
            database::create_project(
                &mut establish_connection(),
                &self.project_name,
                self.path.to_str().unwrap(),
                &self.language.name,
            );
            Success::new(
                format!("Successfully created project {}.", project.name),
                crate::ui::SuccessType::Database,
            )
        } else {
            // database::update_todo(UpdateTodo {
            //     id: &project.id,
            //     project_id: &project.project_id,
            //     title: &project.title,
            //     subtitle: Some(&project.subtitle),
            //     content: Some(&project.content),
            // });
            Success::new(
                format!(
                    "Updated todo ({}, id {}) successfully.",
                    project.name, project.id
                ),
                crate::ui::SuccessType::Database,
            )
        };

        Ok(result)
    }
}
const MAX_TRUNC_NAME: usize = 28;
impl crate::ui::WindowUI for ProjectEditor {
    fn name_truncated(&self) -> String {
        let mut name = self.name.clone();
        name.truncate(MAX_TRUNC_NAME);
        if self.name.len() > name.len() {
            name += "...";
        }
        name
    }

    fn _show(&mut self, ctx: &egui::Context, open: &mut bool) {
        use crate::ui::View as _;
        egui::Window::new(self.name_truncated())
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| self.ui(ui));
    }
}
impl crate::ui::View for ProjectEditor {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            project_name,
            language,
            back_up: _,
            path,
            id: _,
            gid: _,
            path_backup: _,
            name: _,
            modified: _,
        } = self;
        ui.horizontal(|ui| {
            ui.set_height(0.0);
            ui.label("Your todo...");
        });

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Title");
            // ui.text_edit_singleline(path.into());
            ui.label("Subtitle");
            ui.text_edit_singleline(project_name);
            // ui.label("Compile the demo with the ");
            // ui.code("syntax_highlighting");
            // ui.label(" feature to enable more accurate syntax highlighting using ");
            // ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
            // ui.label(".");
        });

        let mut theme =
            egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        // let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
        //     let mut layout_job = egui_extras::syntax_highlighting::highlight(
        //         ui.ctx(),
        //         ui.style(),
        //         &theme,
        //         string,
        //         language,
        //     );
        //     layout_job.wrap.max_width = wrap_width;
        //     ui.fonts(|f| f.layout_job(layout_job))
        // };

        // egui::ScrollArea::vertical().show(ui, |ui| {
        //     ui.add(
        //         egui::TextEdit::multiline(code)
        //             .font(egui::TextStyle::Monospace) // for cursor height
        //             .code_editor()
        //             .desired_rows(13)
        //             .lock_focus(true)
        //             .desired_width(f32::INFINITY)
        //             .layouter(&mut layouter),
        //     );
        // });

        self.update_name();
    }
}

impl From<Project> for ProjectEditor {
    fn from(value: crate::models::Project) -> Self {
        Self::new(
            &PathBuf::from(value.path),
            &value.name,
            &get_language(&value.id),
            StoredId::Stored(value.id),
        )
    }
}
impl From<FormattedProject> for ProjectEditor {
    fn from(value: FormattedProject) -> Self {
        let id = if value.new {
            StoredId::New(value.id)
        } else {
            StoredId::Stored(value.id)
        };
        Self::new(
            &PathBuf::from(value.path),
            &value.name,
            &get_language(&value.id),
            id,
        )
    }
}

impl From<&mut ProjectEditor> for FormattedProject {
    fn from(value: &mut ProjectEditor) -> Self {
        let (new, id) = match value.id {
            StoredId::New(id) => (true, id),
            StoredId::Stored(id) => (false, id),
        };
        Self {
            id,
            language_id: value.language.id,
            name: value.project_name.clone(),
            path: value
                .path
                .to_str()
                .expect("If this failed, your path is not convertible to utf8.")
                .into(),
            new,
        }
    }
}

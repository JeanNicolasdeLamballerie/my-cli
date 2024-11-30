//

use egui::{Color32, Pos2, RichText, ScrollArea, SidePanel, Ui};
use tabled::settings::panel::VerticalPanel;

use crate::{
    database::{self, Save},
    editor::{Modified, TodoEditor, WindowUI},
    models::{FormattedTodo, NewTodo, Project, ProjectWithLanguageName, UpdateTodo},
    schema::todos::subtitle,
    ui::{DatabaseError, Feedback, Log, Success},
};

impl From<TodoEditor> for FormattedTodo {
    fn from(value: TodoEditor) -> Self {
        let (id, is_new) = match value.id {
            TodoId::Stored(id) => (id, false),
            TodoId::New(id) => (id, true),
        };
        FormattedTodo {
            id,
            title: value.title,
            subtitle: value.subtitle,
            content: value.code,
            project_id: value.project_id,
            new: is_new,
        }
    }
}
impl From<&mut TodoEditor> for FormattedTodo {
    fn from(value: &mut TodoEditor) -> Self {
        let (id, is_new) = match value.id {
            TodoId::Stored(id) => (id, false),
            TodoId::New(id) => (id, true),
        };
        FormattedTodo {
            id,
            title: value.title.clone(),
            subtitle: value.subtitle.clone(),
            content: value.code.clone(),
            project_id: value.project_id,
            new: is_new,
        }
    }
}
// impl TryFrom<FormattedTodo> for NewTodo<'_> {
//     type Error = ();

//     fn try_from(value: FormattedTodo) -> Result<Self, Self::Error> {
//         Ok(Self {
//             title: &value.title,
//             subtitle: Some(&value.subtitle),
//             content: Some(&value.content),
//             project_id: &value.project_id,
//         })
//     }
// }

impl Into<TodoEditor> for FormattedTodo {
    fn into(self) -> TodoEditor {
        let id = if self.new {
            TodoId::New(self.id)
        } else {
            TodoId::Stored(self.id)
        };
        TodoEditor::new(
            "md",
            &self.title,
            &self.subtitle,
            &self.content,
            id,
            self.project_id,
        )
    }
}
#[derive(Clone, Debug)]
pub enum TodoId {
    New(i32),
    Stored(i32),
}

// TODO add slug
pub struct Tag {
    //todo
}

// pub struct Todo {

// }

// impl Todo {
//     pub fn new(title: String, subtitle: String, content: String) {
//         return Self {
//             title,
//             subtitle,
//             content,
//             tags:None,
//         };
//     }
// }
// pub trait View {
//     fn ui(&mut self, ui: &mut egui::Ui);
// }

// /// Something to view
// pub trait WindowUI {
//     /// `&'static` so we can also use it as a key to store open/close state.
//     fn name(&self) -> &str;

//     /// Show windows, etc
//     fn show(&mut self, ctx: &egui::Context, open: &mut bool);
// }
#[derive(Debug)]
pub enum TodoListState {
    VIEW,
    EDIT,
}

pub struct TodoList {
    todos: Vec<TodoEditor>,
    new_todos: i32,
    parent: ProjectWithLanguageName,
    state: TodoListState,
    target: Option<TodoEditor>,
    log: Log<Result<Success, DatabaseError>>,
}

impl Default for TodoList {
    fn default() -> Self {
        let parent = ProjectWithLanguageName::new((
            Project {
                id: 0,
                language_id: 0,
                name: "Project Name".into(),
                path: "/some/path/.".into(),
            },
            "Some Language".into(),
        ));
        Self {
            parent,
            todos: Vec::from([
                // TodoEditor::default(),
                // TodoEditor::id_default(1),
                // TodoEditor::id_default(2),
                // TodoEditor::id_default(3),
                // TodoEditor::id_default(4),
                // TodoEditor::id_default(5),
                // TodoEditor::id_default(6),
                // TodoEditor::id_default(7),
                // TodoEditor::id_default(8),
                // TodoEditor::id_default(9),
                // TodoEditor::id_default(10),
                // TodoEditor::id_default(11),
            ]),
            new_todos: 0,
            state: TodoListState::EDIT,
            target: None,
            log: Log::new(Vec::from([
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success with a very long string should look like this ?sdifjizejij hello !"
                        .into(),
                    crate::ui::SuccessType::Database,
                )),
                Err(DatabaseError::new("Error occured x y z")),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success !".into(),
                    crate::ui::SuccessType::Database,
                )),
                Ok(Success::new(
                    "success 2 !".into(),
                    crate::ui::SuccessType::Database,
                )),
            ])),
        }
    }
}

impl eframe::App for TodoList {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use crate::editor::View as _;

        // let size = ctx.available_rect();

        egui::TopBottomPanel::top(egui::Id::new("top_panel_todo_list"))
            // .max_height(size.height() / 10.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Project Information").size(25.));
                });
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::Vec2 { x: 500., y: 500. },
                        egui::Layout::top_down(egui::Align::LEFT),
                        |ui| {
                            ui.label(String::from("Project Name : ") + &self.parent.name);
                            ui.label(String::from("Project Path : ") + &self.parent.path);
                            ui.label(
                                String::from("Project Language : ") + &self.parent.language_name,
                            );
                            ui.separator();
                            ui.label(
                                RichText::new(format!("Number of Todos : {}", self.todos.len()))
                                    .color(Color32::LIGHT_GRAY)
                                    .background_color(Color32::BLACK),
                            );
                            ui.separator();
                            match self.state {
                                TodoListState::EDIT => {
                                    if ui.button("New Todo").clicked() {
                                        self.new_todos += 1;
                                        self.todos.push(TodoEditor::new(
                                            "md",
                                            "",
                                            "",
                                            "",
                                            TodoId::New(self.new_todos),
                                            self.parent.id,
                                        ))
                                    }
                                    if ui.button("Save all").clicked() {
                                        self.save_to_db().unwrap();
                                    }
                                }

                                _ => {
                                    if ui.button("Swap to Edit").clicked() {
                                        self.state = TodoListState::EDIT;
                                    }
                                }
                            }
                        },
                    );
                    // ui.add(egui::Separator::default());
                    ui.separator();
                    ScrollArea::vertical().show(ui, |ui| {
                        self.log.process(ui);
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            TodoListState::VIEW => self.display(ui),
            TodoListState::EDIT => self.ui(ui),
        });
    }
}
// impl crate::editor::WindowUI for TodoList {
//     fn name(&self) -> &str {
//        "🖮 Todo List"
//     }

//     fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
//         use crate::editor::View as _;
//         egui::Window::new(self.name())
//             .open(open)
//             .default_height(500.0)
//             .show(ctx, |ui| self.ui(ui));
//     }
// }

impl TodoList {
    pub fn display(&mut self, ui: &mut egui::Ui) {
        //TODO move the cache to the list
        let target = self
            .target
            .clone()
            .unwrap_or(TodoEditor::new(
                "md",
                "An Error Occured",
                "Try reloading the app...",
                "**Could not open the todo...**",
                TodoId::New(404),
                self.parent.id,
            ))
            .clone();
        let display = format!("{}\n\n{}", target.title_str(), target.code);

        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut cache, &display);
    }
}

impl crate::editor::View for TodoList {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let used_rectangle = ui.ctx().available_rect();
        let x_delta = 100f32; //px
        let y_delta = 150f32;
        let initial = used_rectangle.min;
        let maximum = used_rectangle.max;
        let mut rows = 0f32;
        let mut columns = 0f32;
        let default_width = 200.0;
        let default_height = 500.0;
        for element in &mut self.todos {
            let pos = if rows * x_delta > maximum.x - default_width {
                columns += 1f32;
                rows = 1f32;
                Pos2::new(initial.x, initial.y + columns * y_delta)
            } else {
                let x = initial.x + x_delta * rows;
                rows += 1f32;
                Pos2::new(x, initial.y + columns * y_delta)
            };

            element.modified = element.is_modified();
            println!("Name / GID : {} for {}", element.name(), element.gid);
            egui::Window::new(element.name())
                .id(egui::Id::new(&element.gid))
                //TODO check for closing
                //.default_open(false)
                .pivot(egui::Align2::LEFT_TOP)
                .default_pos(pos) //default pos is bugged
                .default_width(default_width)
                .max_width(maximum.x)
                .default_height(default_height)
                .max_height(maximum.y)
                .show(ui.ctx(), |ui| {
                    element.ui(ui);
                    if ui.button("Preview").clicked() {
                        self.target = Some(element.clone());
                        self.state = TodoListState::VIEW;
                    }
                    if ui
                        .add_enabled(element.modified, egui::Button::new("Save"))
                        .clicked()
                    {
                        //TODO save to db ?
                    }
                });
        }
    }
}
//TODO error handling
impl crate::database::Save<Vec<FormattedTodo>> for TodoList {
    fn save_to_db(&mut self) -> Result<Success, DatabaseError> {
        let mut insert: Vec<NewTodo> = Vec::new();
        let mut edit = Vec::new();
        let saved = self.to_saved_format();
        for todo in &saved {
            if todo.new {
                insert.push(NewTodo {
                    title: &todo.title,
                    subtitle: Some(&todo.subtitle),
                    content: Some(&todo.content),
                    project_id: &todo.project_id,
                });
            } else {
                edit.push(UpdateTodo {
                    title: &todo.title,
                    subtitle: Some(&todo.subtitle),
                    content: Some(&todo.content),
                    project_id: &todo.project_id,
                    id: &todo.id,
                });
            }
        }
        let rows = database::batch_create_todo(&insert);
        println!("Added {} new todos.", rows);
        let rows_update = database::batch_edit_todo(edit);
        println!("Edited {} todos.", rows_update);
        Ok(Success::new(
            format!(
                "Added {} todos and edited {} todos successfully.",
                rows, rows_update
            ),
            crate::ui::SuccessType::Database,
        ))
    }
    fn to_saved_format(&mut self) -> Vec<FormattedTodo> {
        let mut todos = Vec::new();
        for todo in self.todos.iter_mut() {
            todos.push(todo.to_saved_format())
        }
        todos
    }
}

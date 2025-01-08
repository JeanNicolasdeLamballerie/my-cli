//

use std::io::Write;

use egui::{Align, Color32, Pos2, RichText, ScrollArea, TextStyle};

use crate::{
    database::{self, delete_all_todos, delete_todo, get_todos_for_proj, Save},
    editor::{Modified, TodoEditor},
    models::{FormattedTodo, NewTodo, Project, ProjectWithLanguageName, UpdateTodo},
    ui::{DatabaseError, Log, Success},
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

#[derive(Clone, Debug)]
pub enum TodoId {
    New(i32),
    Stored(i32),
}

// TODO add slug
pub struct Tag {
    //todo
}
#[derive(Debug, Clone)]
pub enum TodoListState {
    VIEW,
    EDIT,
}

pub struct FileTodo {
    filename: String,
    file_content: String,
}
#[derive(Clone)]
pub struct TodoList {
    todos: Vec<TodoEditor>,
    new_todos: i32,
    parent: ProjectWithLanguageName,
    state: TodoListState,
    target: Option<TodoEditor>,
    log: Log<Result<Success, DatabaseError>>,
    refresh: bool,
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
            refresh: false,
            parent,
            todos: Vec::from([]),
            new_todos: 0,
            state: TodoListState::EDIT,
            target: None,
            log: Log::new(Vec::from([Ok(Success::new(
                "Connected to database.".into(),
                crate::ui::SuccessType::Database,
            ))])),
        }
    }
}

impl eframe::App for TodoList {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use crate::ui::View as _;
        egui::TopBottomPanel::top(egui::Id::new("top_panel_todo_list"))
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
                                        let r = self.save_to_db();
                                        self.push_log(r);
                                    }
                                    let response = ui.button("Delete all");
                                    let popup_id = ui.make_persistent_id("DELETE_ALL_POPUP");
                                    if response.clicked() {
                                        ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                    }
                                    let below = egui::AboveOrBelow::Below;
                                    let ignore_clicks =
                                        egui::popup::PopupCloseBehavior::IgnoreClicks;
                                    egui::popup::popup_above_or_below_widget(
                                        ui,
                                        popup_id,
                                        &response,
                                        below,
                                        ignore_clicks,
                                        |ui| {
                                            ui.set_min_size(egui::Vec2 { x: 400.0, y: 180.0 });
                                            ui.heading("This operation is not recoverable. You will delete every todo created for this project.");
                                            ui.separator();
                                            if ui.button("Confirm deletion").clicked() {
                                                delete_all_todos(&self.parent.id);
                                                // FIXME make this have a confirmation screen.
                                                let r = Ok(Success::new(
                                                    format!("Deleted all todos for this project.",),
                                                    crate::ui::SuccessType::Database,
                                                ));
                                                self.push_log(r);
                                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                            }
                                            if ui.button("Cancel").clicked() {
                                                ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                                            }
                                        },
                                    );
                                    let response_file = ui.add_enabled(self.todos.len() > 0, egui::Button::new("Generate todo files"));

                                    let file_popup_id = ui.make_persistent_id("GENERATE_FILES_POPUP");
                                   if response_file.clicked {
                                        ui.memory_mut(|mem| mem.toggle_popup(file_popup_id));
                                    }
                                    egui::popup::popup_above_or_below_widget(
                                        ui,
                                        file_popup_id,
                                        &response_file,
                                        below,
                                        ignore_clicks,
                                        |ui| {
                                            ui.set_min_size(egui::Vec2 { x: 400.0, y: 180.0 });
                                            ui.heading("Generate todo files :");
                                            ui.separator();
                                            // ui.checkbox(checked, text)
                                            if ui.button("Generate file from all todos").clicked() {
                                                let r = self.del_file();
                                                self.push_log(r);
                                                ui.memory_mut(|mem| mem.toggle_popup(file_popup_id));
                                            }
                                            if ui.button("Cancel").clicked() {
                                                ui.memory_mut(|mem| mem.toggle_popup(file_popup_id));
                                            }
                                        },
                                    );
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
                        self.log.ui(ui);
                    });
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            TodoListState::VIEW => self.display(ui),
            TodoListState::EDIT => self.ui(ui),
        });
    }
}

impl From<&TodoList> for FileTodo {
    fn from(value: &TodoList) -> Self {
        let mut file_str = String::new();
        for todo in &value.todos {
            file_str.push_str(&todo.to_display());
            file_str.push('\n');
        }

        Self {
            filename: "todos.md".into(),
            file_content: file_str,
        }
    }
}

impl From<&TodoEditor> for FileTodo {
    fn from(value: &TodoEditor) -> Self {
        Self {
            filename: String::new() + "todo-" + &value.title + ".md",
            file_content: value.to_display(),
        }
    }
}

impl TodoList {
    pub fn del_file(&mut self) -> Result<Success, DatabaseError> {
        let clone = &self.clone();
        let file: FileTodo = clone.into();
        let file_path = std::path::PathBuf::from(&self.parent.path);
        match file_path.canonicalize() {
            Ok(mut path) => {
                //FIXME Delete file if it exists. Also, no
                //unwrapping here
                path.push(file.filename);
                match path.canonicalize() {
                    Ok(p) => {
                        let removed = std::fs::remove_file(p);
                        if removed.is_err() {
                            //FIXME make Error an enum
                            //with either db or file
                            return Err(DatabaseError::new(&removed.unwrap_err().to_string()));
                        }
                    }
                    Err(_) => (),
                };
                let mut file_handle = std::fs::File::create_new(path).unwrap();
                file_handle
                    .write_all(&file.file_content.as_bytes())
                    .unwrap();
                Ok(Success::new(
                    format!("Generated file for all todos for this project.",),
                    crate::ui::SuccessType::Database,
                ))
            }
            Err(err) => Err(DatabaseError::new(&err.to_string())),
        }
    }
    pub fn to_file(&self, target: Option<Vec<TodoEditor>>) -> FileTodo {
        match target {
            Some(todos) => {
                let mut display_str = String::new();
                for todo in &todos {
                    display_str.push_str(&(todo.to_display() + "\n"));
                }
                return FileTodo {
                    filename: "todos.md".into(),
                    file_content: display_str,
                };
            }
            None => self.into(),
        }
    }
    pub fn to_multiple_files(&self, target: Option<Vec<TodoEditor>>) -> Vec<FileTodo> {
        todo!();
    }
    pub fn push_log(&mut self, result: Result<Success, DatabaseError>) {
        self.log.push(result);
        self.log.should_scroll();
        self.refresh = true;
    }
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
        let display = target.to_display();

        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut cache, &display);
    }
    pub fn add(&mut self, todo: TodoEditor) {
        self.todos.push(todo);
    }
    pub fn retrieve(&mut self) {
        let todos = get_todos_for_proj(self.parent.id);

        // FIXME Might need to instead merge the two vecs with rules on how to handle.
        // lets just replace it for now.
        self.todos = todos.into_iter().map(|todo| todo.into()).collect();
    }

    pub fn with_parent(&mut self, parent: &ProjectWithLanguageName) -> &mut Self {
        self.parent = parent.clone();
        self
    }
}

impl crate::ui::View for TodoList {
    fn ui(&mut self, ui: &mut egui::Ui) {
        use crate::ui::WindowUI as _;
        if self.refresh {
            self.refresh = false;
            self.retrieve();
        }
        let used_rectangle = ui.ctx().available_rect();
        let x_delta = 100f32; //px
        let y_delta = 150f32;
        let initial = used_rectangle.min;
        let maximum = used_rectangle.max;
        let mut rows = 0f32;
        let mut columns = 0f32;
        let default_width = 400.0;
        let default_height = 500.0;
        let mut added_logs: Vec<Result<Success, DatabaseError>> = Vec::new();
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
                        let r = element.save_to_db();
                        added_logs.push(r);
                    }
                    if ui.button("Delete").clicked() {
                        match element.id {
                            TodoId::New(_) => todo!(),
                            TodoId::Stored(id) => {
                                delete_todo(&id);
                                let r = Ok(Success::new(
                                    format!(
                                        "Deleted Todo ({}, titled {} ) successfully.",
                                        id, element.title
                                    ),
                                    crate::ui::SuccessType::Database,
                                ));

                                added_logs.push(r);
                            }
                        }
                    }
                });
        }
        if added_logs.len() > 0 {
            for log in added_logs {
                self.push_log(log);
            }
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
        let rows_update = database::batch_edit_todo(edit);
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

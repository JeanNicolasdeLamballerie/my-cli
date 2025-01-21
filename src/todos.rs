//

use std::{io::Write, ops::Deref, path, str::FromStr};

use egui::{ahash::HashMapExt, Align, Color32, Pos2, RichText, ScrollArea, TextStyle};

use crate::{
    database::{self, delete_all_todos, delete_todo, get_todos_for_proj, Save},
    editor::{Modified, TodoEditor},
    models::{FormattedTodo, NewTodo, Project, ProjectWithLanguageName, UpdateTodo},
    ui::{DatabaseError, Log, Success,  WindowUI},
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

#[derive(Clone)]
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
    remove: Vec<usize>,
    multiple_files: MultipleFiles,
}
#[derive(Default, Clone)]
struct MultipleFiles {
    selected: bool,
    files: Option<Vec<FileTodo>>,
    selection: Vec<bool>,
    target_path : Option<std::path::PathBuf>,
    path: String,
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
            multiple_files: MultipleFiles::default(),
            refresh: false,
            parent,
            todos: Vec::from([]),
            new_todos: 0,
            state: TodoListState::EDIT,
            target: None,
            remove: Vec::new(),
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
                                            let min_rectangle = egui::Vec2 { x: 300.0, y: 80.0 };
                                            ui.set_min_size(min_rectangle);
                                            ui.allocate_ui(min_rectangle, |ui| {

                                                ui.horizontal(|ui|{
                                                ui.allocate_space(egui::vec2(15.0, 20.));
                                                let text = RichText::new("This operation is not recoverable. You will delete every todo created for this project.").heading().color(Color32::BLACK);
                                                let title =  egui::Label::new(text.clone()).wrap();
                                                let title_rect =   title.layout_in_ui(ui).2.rect;
                                                ui.allocate_space(egui::vec2(15.0, 20.));
                                                let full_heading_size =ui.min_rect();
                                                ui.painter().rect_filled(full_heading_size, 5.0, Color32::DARK_RED);
                                                ui.put(title_rect, egui::Label::new(text).wrap());
                                            });

                                                });
                                            ui.separator();
                                            if ui.button("Confirm deletion").clicked() {
                                                delete_all_todos(&self.parent.id);
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
                                            let (x, y) = (400.0, 180.0);
                                            let max_rectangle = egui::Vec2 { x, y };
                                            // let visuals = egui::Visuals { panel_fill: egui::Color32::RED, ..Default::default() };
                                            ui.set_min_size(max_rectangle);
                                            ui.allocate_ui(max_rectangle, |ui| {
                                            let text = RichText::new("Generate todo files :").heading().color(Color32::BLACK);
                                           let title =  egui::Label::new(text.clone());
                                                title.layout_in_ui(ui);
                                                let r = ui.min_rect();
                                                ui.painter().rect_filled(r, 5.0, Color32::DARK_GREEN);
                                                ui.put(r, egui::Label::new(text));
                                            });
                                            ui.separator();
                                            ui.vertical_centered(|ui| {

                                           let e = ui.horizontal(|ui| {
                                            if ui.button("Generate file from all todos").clicked() {
                                                let r = self.replace_file();
                                                self.push_log(r);
                                                ui.memory_mut(|mem| mem.toggle_popup(file_popup_id));
                                            };
                                            ui.separator();
                                            if ui.button("Generate multiple files").clicked() {
                                                let path = &mut self.multiple_files.path;
                                                if path == "" {
                                                let parent_path = self.parent.path.clone();
                                                let mut p = std::path::PathBuf::from(&parent_path).canonicalize().unwrap();
                                                    p.push("todos");                                                    
                                                            *path = p.to_str().unwrap_or("error loading the path...").into();
                                                }
                                                self.multiple_files.selected = true;
                                            };
                                            });
                                            });
                                            if self.multiple_files.selection.len() != self.todos.len() {
                                                self.multiple_files.selection = vec![false;self.todos.len()];
                                                self.multiple_files.files = None;
                                            }
                                            if self.multiple_files.selected {
                                                use crate::ui::WindowUI as _;
                                                for (idx, todo) in self.todos.iter().enumerate() {
                                                    let checked = &mut self.multiple_files.selection[idx];
                                                    ui.checkbox(checked, todo.name());
                                                }

                                                ui.separator();
                                                let path = &mut self.multiple_files.path;
                                                let parent_path = self.parent.path.clone();
                                                let mut p = std::path::PathBuf::from(&parent_path).canonicalize().unwrap();
                                                p.push("todos");    
                                               let hint = RichText::new(p.to_str().unwrap_or("error loading the path...")).color(Color32::DARK_GRAY);

                                                ui.label("The path to save your todos (defaults to your project's directory with a 'todo' folder) :");
                                                egui::TextEdit::singleline(path).hint_text(hint).show(ui);
                                                if ui.add_enabled(self.multiple_files.selection.iter().any(|&x| x), egui::Button::new("generate multiple files")).clicked(){
                                                    let mut todos = Vec::new();
                                                    for (idx, added) in self.multiple_files.selection.iter().enumerate() {
                                                        if *added {
                                                            todos.push(self.todos[idx].clone());
                                                        }
                                                    }
                                                     
                                                    let base_path = std::path::PathBuf::from(&parent_path).canonicalize().unwrap();
                                                    let res =  handle_path(std::path::Path::new(&path).to_path_buf(), base_path);
                                    match res {

                                                     Ok(target_path) => {
                                                            self.multiple_files.target_path = Some(target_path);
                                                            self.multiple_files.files = Some(to_multiple_files(todos));
                                                        },
                                                        Err(error) => {
                                                            self.push_log(Err(error));
                                                        }
                                                    }
                                                };
                                                if let Some(files) = &self.multiple_files.files  {
                                                    if let Some(verified_path) = &self.multiple_files.target_path {
                                                    ui.separator();
                                                    for file in files.iter() {
                                                      let mut target = verified_path.clone();
                                                        target.push(&file.filename);
                                                        let txt = RichText::new(target.to_str().unwrap_or("error creating the path...")).small();
                                                        ui.label(txt);
                                                        ui.separator();
                                                    }
                                                    if ui.button("Save all").clicked() {
                                                        if let Some(path) = &self.multiple_files.target_path {
                                                                if let Some(files) = &self.multiple_files.files {
                                                                   let logs = create_files(files, path);
                                                                    for log in logs {
                                                                        self.push_log(log);
                                                                    }
                                                                }
                                                            }
                                                            self.multiple_files = MultipleFiles::default();
                                                            ui.memory_mut(|mem| mem.toggle_popup(file_popup_id));
                                                        }
                                                    }
                                                }
                                            }
                                            ui.separator();
                                            if ui.button("Cancel").clicked() {
                                                self.multiple_files = MultipleFiles::default();
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
    pub fn replace_file(&mut self) -> Result<Success, DatabaseError> {
        let clone = &self.clone();
        let file: FileTodo = clone.into();
        let file_path = std::path::PathBuf::from(&self.parent.path);
        match file_path.canonicalize() {
            Ok(mut path) => {
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
                // FIXME remove unwrap here for file errors. Do this when a proper error enum
                // exists
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
        let mut added: Vec<TodoEditor> = self
            .todos
            .clone()
            .into_iter()
            .filter(|td| match td.id {
                TodoId::New(_) => true,
                TodoId::Stored(_) => false,
            })
            .collect();
        let mut tds: Vec<TodoEditor> = todos.into_iter().map(|todo| todo.into()).collect();
        tds.append(&mut added);
        self.todos = tds;
    }

    pub fn with_parent(&mut self, parent: &ProjectWithLanguageName) -> &mut Self {
        self.parent = parent.clone();
        self
    }
}

impl crate::ui::View for TodoList {
    fn ui(&mut self, ui: &mut egui::Ui) {
        use crate::ui::WindowUI as _;
        if self.remove.len() > 0 {
            for &idx in &self.remove {
                self.todos.remove(idx);
            }
            //FIXME Could be optimized by consuming, emptying and reusing the array instead.
            self.remove = Vec::new();
        }
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
        let mut todo_index = 0;
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
                //FIXME use the trait ("element.show()")
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

                    match element.id {
                        TodoId::New(id) => {
                            if ui.button("discard").clicked() {
                                self.remove.push(todo_index);
                                let r = Ok(Success::new(
                                    format!(
                                        "Discarded new Todo ({}, titled {} ) successfully.",
                                        id, element.title
                                    ),
                                    crate::ui::SuccessType::Database,
                                ));
                                added_logs.push(r);
                            }
                        }
                        TodoId::Stored(id) => {
                            if ui.button("delete").clicked() {
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
                    };
                });
            todo_index += 1;
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
fn to_multiple_files(target: Vec<TodoEditor>) -> Vec<FileTodo> {
    let mut map: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    let mut files: Vec<FileTodo> = Vec::new();
    for todo in target.iter() {
        let mut file: FileTodo = todo.into();
        let title: &str = &todo.title;
        if let Some(v) = map.get_mut(title) {
            *v += 1;
            file.filename = format!("{title}-({}).md", v);
        } else {
            map.insert(&todo.title, 0);
            file.filename = format!("{}.md", title);
        }
        files.push(file);
    }
    files
}

fn handle_path<Path>(path: Path, base_path: Path) -> Result<std::path::PathBuf, DatabaseError>
where
    Path: AsRef<std::path::Path>,
{
    use std::path;
    let path: &path::Path = path.as_ref();
    let base_path: &path::Path = base_path.as_ref();
    let full_path = if path.is_relative() {
        base_path.join(path)
    } else {
        path.to_path_buf()
    };
    let mut parent: Option<&path::Path> = Some(&full_path);
    let tmp_clone = full_path.clone();
    let mut iterator = tmp_clone.components().rev();
    let mut leftovers = Vec::new();
    let result = loop {
        match parent {
            Some(path) => match path.canonicalize() {
                Ok(usable_path) => {
                    let path = establish_path(leftovers, usable_path);
                    // let result = Ok(Success::new(
                    //     format!("Created path at target : '{}'.", path),
                    //     crate::ui::SuccessType::Database,
                    // ));
                    break Ok(path);
                }
                Err(_) => {
                    let last = iterator.next();
                    leftovers.push(last);
                    parent = path.parent();
                }
            },
            None => {
                let result = Err(DatabaseError::new(&format!("The path was fully unwinded and couldn't find a correct base directory(e.g, a drive).")));
                break result;
            }
        }
    };
    return result;
}
fn establish_path<Path>(
    leftovers: Vec<Option<path::Component>>,
    canonicalized_path: Path,
) -> std::path::PathBuf
where
    Path: AsRef<std::path::Path>,
{
    use std::path;
    let canonicalized_path = canonicalized_path.as_ref().to_path_buf();
    let mut create = path::PathBuf::new();
    for element in leftovers.iter().rev() {
        if let Some(component) = element {
            create.push(component);
        }
    }
    let final_path = canonicalized_path.join(create);
    final_path
}

fn create_files<Path>(files:&[FileTodo], path: Path) -> Vec<Result<Success, DatabaseError>> where Path: AsRef<std::path::Path> {
    use crate::ui::SuccessType;

    let result = std::fs::create_dir_all(&path);
    let mut logs : Vec<Result<Success, DatabaseError>> = Vec::new();
    match result {
        Ok(_) => {
            logs.push(Ok(Success::new(format!("The directory {} was successfully created.", path.as_ref().display()), SuccessType::File)));
            for file in files.iter() {
                let path = path.as_ref().to_path_buf().join(&file.filename);
                  match path.canonicalize() {
                    Ok(p) => {
                        let removed = std::fs::remove_file(p);
                        if removed.is_err() {
                            //FIXME make Error an enum
                            //with either db or file
                            logs.push(Err(DatabaseError::new(&removed.unwrap_err().to_string())));
                            continue;
                        }
                    }
                    Err(_) => (),
                };
                let file_handle = std::fs::File::create_new(&path);
                if let Err(err) = &file_handle {
                    logs.push(Err(DatabaseError::new(&format!("An error occured creating the file : {}", &err.to_string()))));
                    continue;
                }
                match file_handle.unwrap()
                    .write_all(&file.file_content.as_bytes()) {
                        Ok(_) => {
                        logs.push(Ok(Success::new(
                    format!("Generated file {} for for this project.", &file.filename),
                    SuccessType::File,
                )));
                    },
                    Err(err) => {
                        logs.push(Err(DatabaseError::new(&err.to_string())));
                    
                    }
                };
            }
        }
        Err(err) => {
            logs.push(Err(DatabaseError::new(&err.to_string())));
        }
    };
    logs
}

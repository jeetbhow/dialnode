//! Top-level app state: the repository screen and the editor.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use eframe::egui::{
    self, Align, Frame, Id, Key, KeyboardShortcut, Layout, Modal, Modifiers, RichText, ScrollArea,
    Sense, Stroke, TextEdit, TextureHandle, TextureOptions, Ui, ViewportCommand, vec2,
};

use crate::database_modal::{DatabaseModal, EntityRequest, Fulfilled};
use crate::graph_editor::{GraphEnv, GraphEvent, GraphView};
use crate::images::{self, TextureCache};
use crate::model::{DialogueTree, Entities, Graph, NodeData, ROOT_ID, Repository, TreeItemKind};
use crate::repo_screen::RepoScreen;
use crate::repository::{self, OpenRepository};
use crate::storage;
use crate::theme;

const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

pub struct DialnodeApp {
    screen: Screen,
    logo: Option<TextureHandle>,
    confirm: Option<PendingExit>,
    allow_close: bool,
    title: String,
    #[cfg(debug_assertions)]
    debug: crate::debug_hooks::DebugHooks,
}

enum Screen {
    Repository(RepoScreen),
    Editor(Box<Editor>),
}

#[derive(Clone, Copy)]
enum PendingExit {
    Quit,
    ManageRepositories,
}

impl DialnodeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let logo = images::decode(include_bytes!("../assets/logo.png")).map(|img| {
            cc.egui_ctx
                .load_texture("logo", img, TextureOptions::LINEAR)
        });
        #[allow(unused_mut)]
        let mut screen = Screen::Repository(RepoScreen::default());
        #[cfg(debug_assertions)]
        if let Some(path) = crate::debug_hooks::DebugHooks::repository_to_open() {
            match repository::open(&path).and_then(Editor::load) {
                Ok(mut editor) => {
                    editor.open_dialogue = editor.tree.dialogues().into_iter().next();
                    editor.open_folders.extend(
                        editor
                            .tree
                            .items
                            .iter()
                            .filter(|i| i.is_folder())
                            .map(|i| i.id.clone()),
                    );
                    screen = Screen::Editor(Box::new(editor));
                }
                Err(err) => eprintln!("DIALNODE_OPEN failed: {err:#}"),
            }
        }
        Self {
            screen,
            logo,
            confirm: None,
            allow_close: false,
            title: String::new(),
            #[cfg(debug_assertions)]
            debug: crate::debug_hooks::DebugHooks::from_env(),
        }
    }

    fn proceed(&mut self, ctx: &egui::Context, exit: PendingExit) {
        match exit {
            PendingExit::Quit => {
                self.allow_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            PendingExit::ManageRepositories => {
                self.screen = Screen::Repository(RepoScreen::default());
            }
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context, exit: PendingExit) {
        let dirty = matches!(&self.screen, Screen::Editor(e) if e.is_dirty());
        if dirty {
            self.confirm = Some(exit);
        } else {
            self.proceed(ctx, exit);
        }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(exit) = self.confirm else {
            return;
        };
        let mut choice = None;
        Modal::new(Id::unique("unsaved-changes")).show(ctx, |ui| {
            ui.set_width(340.0);
            ui.heading("Unsaved changes");
            ui.label("Save your dialogues before leaving?");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.add(theme::primary_button("Save")).clicked() {
                    choice = Some(true);
                }
                if ui.add(theme::secondary_button("Discard")).clicked() {
                    choice = Some(false);
                }
                if ui.button("Cancel").clicked() {
                    self.confirm = None;
                }
            });
        });
        match choice {
            Some(true) => {
                if let Screen::Editor(editor) = &mut self.screen
                    && !editor.save()
                {
                    // The save error is shown in the editor; stay put.
                    self.confirm = None;
                    return;
                }
                self.confirm = None;
                self.proceed(ctx, exit);
            }
            Some(false) => {
                self.confirm = None;
                self.proceed(ctx, exit);
            }
            None => {}
        }
    }

    fn update_title(&mut self, ctx: &egui::Context) {
        let title = match &self.screen {
            Screen::Repository(_) => "Dialnode".to_string(),
            Screen::Editor(e) => {
                let marker = if e.is_dirty() { " •" } else { "" };
                format!("{}{marker} — Dialnode", e.repo.name)
            }
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }
}

impl eframe::App for DialnodeApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && matches!(&self.screen, Screen::Editor(e) if e.is_dirty())
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm = Some(PendingExit::Quit);
        }

        match &mut self.screen {
            Screen::Repository(screen) => {
                if let Some(open) = screen.ui(ui, self.logo.as_ref()) {
                    match Editor::load(open) {
                        Ok(editor) => self.screen = Screen::Editor(Box::new(editor)),
                        Err(err) => screen.error = Some(format!("{err:#}")),
                    }
                }
            }
            Screen::Editor(editor) => match editor.ui(ui, self.confirm.is_none()) {
                EditorAction::None => {}
                EditorAction::ManageRepositories => {
                    self.request_exit(&ctx, PendingExit::ManageRepositories)
                }
                EditorAction::Quit => self.request_exit(&ctx, PendingExit::Quit),
            },
        }

        self.confirm_dialog(&ctx);
        self.update_title(&ctx);
        #[cfg(debug_assertions)]
        self.debug.after_frame(&ctx);
    }
}

/// One sidebar row. `folder_open` is `Some` for folders and draws a
/// disclosure triangle.
fn tree_row(
    ui: &mut Ui,
    name: &str,
    indent: f32,
    selected: bool,
    folder_open: Option<bool>,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click_and_drag());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let painter = ui.painter();
    if selected {
        painter.rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
    } else if resp.hovered() {
        painter.rect_filled(rect, 4.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let text_color = ui.visuals().text_color();
    let mut x = rect.left() + 8.0 + indent;
    let cy = rect.center().y;
    if let Some(open) = folder_open {
        let points = if open {
            vec![
                egui::pos2(x, cy - 3.0),
                egui::pos2(x + 8.0, cy - 3.0),
                egui::pos2(x + 4.0, cy + 3.0),
            ]
        } else {
            vec![
                egui::pos2(x + 1.0, cy - 4.0),
                egui::pos2(x + 7.0, cy),
                egui::pos2(x + 1.0, cy + 4.0),
            ]
        };
        painter.add(egui::Shape::convex_polygon(
            points,
            text_color,
            Stroke::NONE,
        ));
    }
    x += 16.0;
    let font = egui::FontId::proportional(14.0);
    let text = if folder_open.is_some() {
        RichText::new(name).strong()
    } else {
        RichText::new(name)
    };
    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Truncate),
        rect.right() - x - 4.0,
        font,
    );
    painter.galley(
        egui::pos2(x, cy - galley.size().y / 2.0),
        galley,
        text_color,
    );
    resp
}

pub enum EditorAction {
    None,
    ManageRepositories,
    Quit,
}

enum TreeAction {
    Select(String),
    ToggleFolder(String),
    StartRename(String),
    CommitRename,
    CancelRename,
    /// Asks before deleting.
    Delete(String),
    DeleteConfirmed(String),
    Move(String, String),
    Add {
        parent: String,
        folder: bool,
    },
}

pub struct Editor {
    repo: Repository,
    /// The repository directory.
    root: PathBuf,
    tree: DialogueTree,
    entities: Entities,
    /// The highlighted sidebar item (folder or dialogue).
    selected: Option<String>,
    /// The dialogue shown on the canvas.
    open_dialogue: Option<String>,
    renaming: Option<(String, String)>,
    focus_rename: bool,
    confirm_delete: Option<String>,
    open_folders: HashSet<String>,
    views: HashMap<String, GraphView>,
    db_modal: Option<DatabaseModal>,
    textures: TextureCache,
    /// Dialogues with changes that haven't been written to disk.
    dirty: HashSet<String>,
    status: Option<String>,
    error: Option<String>,
}

impl Editor {
    fn load(open: OpenRepository) -> Result<Self> {
        let OpenRepository { repository, root } = open;
        let (tree, warnings) = storage::load_tree(&root).context("failed to read dialogues")?;
        let entities = storage::load_entities(&root)?;
        let error = (!warnings.is_empty()).then(|| {
            format!(
                "Skipped {} file(s) that aren't valid dialogues: {}",
                warnings.len(),
                warnings.join("; ")
            )
        });
        Ok(Self {
            repo: repository,
            root,
            tree,
            entities,
            selected: None,
            open_dialogue: None,
            renaming: None,
            focus_rename: false,
            confirm_delete: None,
            open_folders: HashSet::new(),
            views: HashMap::new(),
            db_modal: None,
            textures: TextureCache::default(),
            dirty: HashSet::new(),
            status: None,
            error,
        })
    }

    fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Writes every changed dialogue to its file. Returns whether all of
    /// them were written.
    fn save(&mut self) -> bool {
        let ids: Vec<String> = self.dirty.iter().cloned().collect();
        for id in ids {
            let (Some(path), Some(graph)) = (
                storage::path_of(&self.root, &self.tree, &id),
                self.tree.graph(&id),
            ) else {
                self.dirty.remove(&id);
                continue;
            };
            if let Err(err) = storage::write_dialogue(&path, graph) {
                self.error = Some(format!("{err:#}"));
                return false;
            }
            self.dirty.remove(&id);
        }
        self.status = Some("Saved.".into());
        true
    }

    fn export(&mut self) -> Result<()> {
        let Some(id) = self.open_dialogue.clone() else {
            bail!("Open a dialogue to export it.");
        };
        let item = self.tree.get(&id).context("dialogue not found")?;
        let TreeItemKind::Dialogue(graph) = &item.kind else {
            bail!("Only dialogues can be exported.");
        };
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export JSON without layout")
            .set_file_name(format!("{}.json", item.name))
            .add_filter("JSON Files", &["json"])
            .save_file()
        else {
            return Ok(());
        };
        let json = serde_json::to_string_pretty(&storage::dialogue_json(graph, false))?;
        fs::write(&path, json).with_context(|| format!("failed to write {}", path.display()))?;
        self.status = Some(format!("Exported to {}", path.display()));
        Ok(())
    }

    fn ui(&mut self, ui: &mut Ui, interactive: bool) -> EditorAction {
        let mut action = EditorAction::None;
        let ctx = ui.ctx().clone();

        if interactive && ctx.input_mut(|i| i.consume_shortcut(&SAVE_SHORTCUT)) {
            self.save();
        }

        egui::Panel::top("menu-bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Manage Repositories").clicked() {
                        action = EditorAction::ManageRepositories;
                    }
                    let save = egui::Button::new("Save")
                        .shortcut_text(ctx.format_shortcut(&SAVE_SHORTCUT));
                    if ui.add(save).clicked() {
                        self.save();
                    }
                    if ui
                        .add_enabled(self.open_dialogue.is_some(), egui::Button::new("Export…"))
                        .on_hover_text("Write the open dialogue without editor layout")
                        .clicked()
                        && let Err(err) = self.export()
                    {
                        self.error = Some(format!("{err:#}"));
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        action = EditorAction::Quit;
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("New Dialogue").clicked() {
                        self.apply_tree_actions(vec![TreeAction::Add {
                            parent: ROOT_ID.into(),
                            folder: false,
                        }]);
                    }
                    let selected = self.selected.clone();
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Rename"))
                        .clicked()
                        && let Some(id) = selected.clone()
                    {
                        self.apply_tree_actions(vec![TreeAction::StartRename(id)]);
                    }
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Delete"))
                        .clicked()
                        && let Some(id) = selected
                    {
                        self.apply_tree_actions(vec![TreeAction::Delete(id)]);
                    }
                    ui.separator();
                    if ui.button("Open Database").clicked() {
                        self.db_modal = Some(DatabaseModal::new(None));
                    }
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let marker = if self.is_dirty() { " (unsaved)" } else { "" };
                    ui.label(
                        RichText::new(format!("{}{marker}", self.repo.name))
                            .color(theme::SMALL_TEXT),
                    )
                    .on_hover_text(self.root.display().to_string());
                });
            });
        });

        egui::Panel::bottom("status-bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                if let Some(err) = &self.error {
                    ui.colored_label(theme::ERROR, err);
                    if ui.small_button("Dismiss").clicked() {
                        self.error = None;
                    }
                } else if let Some(status) = &self.status {
                    ui.label(RichText::new(status).color(theme::SMALL_TEXT));
                }
            });
        });

        egui::Panel::left("sidebar")
            .resizable(true)
            .default_size(300.0)
            .min_size(220.0)
            .frame(Frame::side_top_panel(ui.style()).fill(theme::SIDEBAR_BG))
            .show(ui, |ui| self.sidebar(ui));

        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas(ui, interactive));

        if let Some(modal) = &mut self.db_modal {
            let out = modal.show(
                &ctx,
                &self.root,
                &mut self.entities,
                &self.repo,
                &mut self.textures,
            );
            if let Some(dir) = out.godot_project {
                self.repo.godot_project_location = Some(dir.to_string_lossy().into_owned());
                self.textures.clear();
                if let Err(err) = repository::save_manifest(&self.repo, &self.root) {
                    self.error = Some(format!("{err:#}"));
                }
            }
            if let Some((request, value)) = out.fulfilled {
                self.fulfill(request, value);
            }
            if out.close {
                self.db_modal = None;
            }
        }

        self.delete_dialog(&ctx);
        action
    }

    fn delete_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_delete.clone() else {
            return;
        };
        let Some(item) = self.tree.get(&id) else {
            self.confirm_delete = None;
            return;
        };
        let what = if item.is_folder() {
            format!("the folder \"{}\" and everything in it", item.name)
        } else {
            format!("\"{}\"", item.file_name())
        };
        let mut confirmed = false;
        let modal = Modal::new(Id::unique("confirm-delete")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading("Delete?");
            ui.label(format!("Move {what} to the trash?"));
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                confirmed = ui.add(theme::primary_button("Move to Trash")).clicked();
                if ui.button("Cancel").clicked() {
                    self.confirm_delete = None;
                }
            });
        });
        if modal.should_close() {
            self.confirm_delete = None;
        }
        if confirmed {
            self.confirm_delete = None;
            self.apply_tree_actions(vec![TreeAction::DeleteConfirmed(id)]);
        }
    }

    fn fulfill(&mut self, request: EntityRequest, value: Fulfilled) {
        let Some(graph) = self.tree.graph_mut(&request.dialogue_id) else {
            return;
        };
        let Some(NodeData::Text(data)) = graph.node_mut(&request.node_id).map(|n| &mut n.data)
        else {
            return;
        };
        match value {
            Fulfilled::Speaker(s) => data.speaker = Some(s.name),
            Fulfilled::Portrait(p) => data.portrait = Some(p.virtual_path),
        }
        if let Some(view) = self.views.get_mut(&request.dialogue_id) {
            view.show_text_field(&request.node_id, request.kind);
        }
        self.dirty.insert(request.dialogue_id);
    }

    fn canvas(&mut self, ui: &mut Ui, interactive: bool) {
        let Some(id) = self.open_dialogue.clone() else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    RichText::new("Select or create a dialogue to start editing.")
                        .color(theme::SMALL_TEXT),
                );
            });
            return;
        };
        let Some(graph) = self.tree.graph_mut(&id) else {
            self.open_dialogue = None;
            return;
        };
        let skills = self.entities.all_skills();
        let view = self.views.entry(id.clone()).or_default();
        let out = view.show(
            ui,
            graph,
            GraphEnv {
                skills: &skills,
                textures: &mut self.textures,
                godot_project: self.repo.godot_project_location.as_deref().map(Path::new),
                keyboard: interactive
                    && self.db_modal.is_none()
                    && self.renaming.is_none()
                    && self.confirm_delete.is_none(),
            },
        );
        if out.changed {
            self.dirty.insert(id.clone());
        }
        for event in out.events {
            match event {
                GraphEvent::RequestEntity { node_id, kind } => {
                    self.db_modal = Some(DatabaseModal::new(Some(EntityRequest {
                        dialogue_id: id.clone(),
                        node_id,
                        kind,
                    })));
                }
                GraphEvent::Error(msg) => self.error = Some(msg),
            }
        }
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        let mut actions = Vec::new();
        let target_folder = self
            .selected
            .as_deref()
            .filter(|id| self.tree.get(id).is_some_and(|i| i.is_folder()))
            .unwrap_or(ROOT_ID)
            .to_string();

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if ui
                .button("+ Dialogue")
                .on_hover_text("New dialogue")
                .clicked()
            {
                actions.push(TreeAction::Add {
                    parent: target_folder.clone(),
                    folder: false,
                });
            }
            if ui.button("+ Folder").on_hover_text("New folder").clicked() {
                actions.push(TreeAction::Add {
                    parent: target_folder.clone(),
                    folder: true,
                });
            }
        });
        ui.separator();

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.tree_level(ui, ROOT_ID, 0, &mut actions);
                // Dropping on the empty space below the list moves to the top level.
                let rest = ui.available_size().max(vec2(ui.available_width(), 40.0));
                let resp = ui.allocate_response(rest, Sense::hover());
                if let Some(id) = resp.dnd_release_payload::<String>() {
                    actions.push(TreeAction::Move((*id).clone(), ROOT_ID.into()));
                }
            });

        self.apply_tree_actions(actions);
    }

    fn tree_level(
        &mut self,
        ui: &mut Ui,
        parent: &str,
        depth: usize,
        actions: &mut Vec<TreeAction>,
    ) {
        let children: Vec<(String, String, bool)> = self
            .tree
            .children(parent)
            .into_iter()
            .map(|i| (i.id.clone(), i.name.clone(), i.is_folder()))
            .collect();

        for (id, name, is_folder) in children {
            let indent = depth as f32 * 16.0;
            if let Some((rename_id, buffer)) = &mut self.renaming
                && *rename_id == id
            {
                ui.horizontal(|ui| {
                    ui.add_space(indent);
                    let resp = ui.add(TextEdit::singleline(buffer).desired_width(f32::INFINITY));
                    if self.focus_rename {
                        resp.request_focus();
                        self.focus_rename = false;
                    }
                    if resp.lost_focus() {
                        if ui.input(|i| i.key_pressed(Key::Enter)) {
                            actions.push(TreeAction::CommitRename);
                        } else {
                            actions.push(TreeAction::CancelRename);
                        }
                    }
                });
                continue;
            }

            let open = self.open_folders.contains(&id);
            let selected = self.selected.as_deref() == Some(id.as_str());
            let label = if !is_folder && self.dirty.contains(&id) {
                format!("{name} •")
            } else {
                name
            };
            let resp = tree_row(ui, &label, indent, selected, is_folder.then_some(open));

            resp.dnd_set_drag_payload(id.clone());
            if resp.clicked() {
                actions.push(TreeAction::Select(id.clone()));
                if is_folder {
                    actions.push(TreeAction::ToggleFolder(id.clone()));
                }
            }
            if resp.double_clicked() {
                actions.push(TreeAction::StartRename(id.clone()));
            }
            if is_folder {
                if resp.dnd_hover_payload::<String>().is_some() {
                    ui.painter().rect_stroke(
                        resp.rect,
                        4.0,
                        Stroke::new(2.0, theme::PRIMARY),
                        egui::StrokeKind::Inside,
                    );
                }
                if let Some(dragged) = resp.dnd_release_payload::<String>() {
                    actions.push(TreeAction::Move((*dragged).clone(), id.clone()));
                }
            }
            resp.context_menu(|ui| {
                if is_folder {
                    if ui.button("New Dialogue Here").clicked() {
                        actions.push(TreeAction::Add {
                            parent: id.clone(),
                            folder: false,
                        });
                    }
                    if ui.button("New Folder Here").clicked() {
                        actions.push(TreeAction::Add {
                            parent: id.clone(),
                            folder: true,
                        });
                    }
                    ui.separator();
                }
                if ui.button("Rename").clicked() {
                    actions.push(TreeAction::StartRename(id.clone()));
                }
                if ui.button("Delete").clicked() {
                    actions.push(TreeAction::Delete(id.clone()));
                }
            });

            if is_folder && open {
                self.tree_level(ui, &id, depth + 1, actions);
            }
        }
    }

    fn apply_tree_actions(&mut self, actions: Vec<TreeAction>) {
        for action in actions {
            if let Err(err) = self.apply_tree_action(action) {
                self.error = Some(format!("{err:#}"));
            }
        }
    }

    /// Applies a sidebar action. Anything that changes the tree changes the
    /// repository directory first, and the tree only if that worked.
    fn apply_tree_action(&mut self, action: TreeAction) -> Result<()> {
        match action {
            TreeAction::Select(id) => {
                if self.tree.graph(&id).is_some() {
                    self.open_dialogue = Some(id.clone());
                }
                self.selected = Some(id);
            }
            TreeAction::ToggleFolder(id) => {
                if !self.open_folders.remove(&id) {
                    self.open_folders.insert(id);
                }
            }
            TreeAction::StartRename(id) => {
                if let Some(item) = self.tree.get(&id) {
                    self.renaming = Some((id, item.name.clone()));
                    self.focus_rename = true;
                }
            }
            TreeAction::CancelRename => self.renaming = None,
            TreeAction::CommitRename => {
                let Some((id, input)) = self.renaming.take() else {
                    return Ok(());
                };
                let item = self.tree.get(&id).context("item not found")?;
                let (folder, parent, old_name) =
                    (item.is_folder(), item.parent.clone(), item.name.clone());
                let name = storage::validate_name(&input, folder, parent == ROOT_ID)?;
                if name == old_name {
                    return Ok(());
                }
                let file_name = if folder {
                    name.clone()
                } else {
                    format!("{name}.json")
                };
                if self.tree.name_taken(&parent, &file_name, Some(&id)) {
                    bail!("Something named \"{file_name}\" already exists here.");
                }
                let from = self.path_of(&id)?;
                self.tree.get_mut(&id).expect("exists").name = name;
                let to = self.path_of(&id)?;
                if let Err(err) = storage::rename(&from, &to) {
                    self.tree.get_mut(&id).expect("exists").name = old_name;
                    return Err(err);
                }
            }
            TreeAction::Move(id, folder) => {
                let Some(item) = self.tree.get(&id) else {
                    return Ok(());
                };
                if item.parent == folder || !self.tree.can_move(&id, &folder) {
                    return Ok(());
                }
                let old_parent = item.parent.clone();
                let file_name = item.file_name();
                if self.tree.name_taken(&folder, &file_name, Some(&id)) {
                    bail!("Something named \"{file_name}\" already exists there.");
                }
                let from = self.path_of(&id)?;
                self.tree.move_item(&id, &folder);
                let to = self.path_of(&id)?;
                if let Err(err) = storage::rename(&from, &to) {
                    self.tree.get_mut(&id).expect("exists").parent = old_parent;
                    return Err(err);
                }
                self.open_folders.insert(folder);
            }
            TreeAction::Delete(id) => self.confirm_delete = Some(id),
            TreeAction::DeleteConfirmed(id) => {
                storage::trash(&self.path_of(&id)?)?;
                self.tree.remove(&id);
                let tree = &self.tree;
                if self
                    .open_dialogue
                    .as_ref()
                    .is_some_and(|d| tree.get(d).is_none())
                {
                    self.open_dialogue = None;
                }
                if self
                    .selected
                    .as_ref()
                    .is_some_and(|s| tree.get(s).is_none())
                {
                    self.selected = None;
                }
                self.views.retain(|d, _| tree.get(d).is_some());
                self.dirty.retain(|d| tree.get(d).is_some());
            }
            TreeAction::Add { parent, folder } => {
                let base = if folder { "New Folder" } else { "Untitled" };
                let name = self.tree.unique_name(&parent, base, folder);
                let parent_dir = if parent == ROOT_ID {
                    self.root.clone()
                } else {
                    self.path_of(&parent)?
                };
                let file_name = if folder {
                    name.clone()
                } else {
                    format!("{name}.json")
                };
                storage::create(&parent_dir.join(file_name), folder)?;

                let kind = if folder {
                    TreeItemKind::Folder
                } else {
                    TreeItemKind::Dialogue(Graph::default())
                };
                let id = self.tree.insert(&parent, name.clone(), kind);
                if parent != ROOT_ID {
                    self.open_folders.insert(parent);
                }
                if !folder {
                    self.open_dialogue = Some(id.clone());
                }
                self.selected = Some(id.clone());
                self.renaming = Some((id, name));
                self.focus_rename = true;
            }
        }
        Ok(())
    }

    fn path_of(&self, id: &str) -> Result<PathBuf> {
        storage::path_of(&self.root, &self.tree, id).context("item not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeType, TextData};
    use eframe::egui::pos2;
    use egui_kittest::Harness;

    fn editor() -> (tempfile::TempDir, Editor) {
        let tmp = tempfile::tempdir().unwrap();
        let open = repository::create("repo", tmp.path(), None).unwrap();
        (tmp, Editor::load(open).unwrap())
    }

    fn add(editor: &mut Editor, parent: &str, folder: bool) -> String {
        editor
            .apply_tree_action(TreeAction::Add {
                parent: parent.into(),
                folder,
            })
            .unwrap();
        editor.renaming = None;
        editor.selected.clone().unwrap()
    }

    fn rename(editor: &mut Editor, id: &str, name: &str) -> Result<()> {
        editor.renaming = Some((id.into(), name.into()));
        editor.apply_tree_action(TreeAction::CommitRename)
    }

    #[test]
    fn sidebar_changes_happen_on_disk() {
        let (_tmp, mut e) = editor();
        let root = e.root.clone();
        let folder = add(&mut e, ROOT_ID, true);
        let dialogue = add(&mut e, &folder, false);
        assert!(root.join("New Folder/Untitled.json").is_file());

        rename(&mut e, &folder, "Chapter 1").unwrap();
        rename(&mut e, &dialogue, "Intro.json").unwrap();
        assert!(root.join("Chapter 1/Intro.json").is_file());
        assert!(!root.join("New Folder").exists());

        e.apply_tree_action(TreeAction::Move(dialogue.clone(), ROOT_ID.into()))
            .unwrap();
        assert!(root.join("Intro.json").is_file());

        let other = add(&mut e, ROOT_ID, false);
        assert!(rename(&mut e, &other, "intro").is_err(), "name clash");
        assert!(rename(&mut e, &other, "a/b").is_err());
        assert!(rename(&mut e, &other, "database").is_err());
        assert!(
            root.join("Untitled.json").is_file(),
            "failed renames leave the file alone"
        );

        // Reloading from disk gives the same tree shape.
        let (reloaded, warnings) = storage::load_tree(&root).unwrap();
        assert!(warnings.is_empty());
        let mut names: Vec<String> = reloaded
            .items
            .iter()
            .map(|i| reloaded.rel_path(&i.id).unwrap().display().to_string())
            .collect();
        names.sort();
        assert_eq!(names, ["Chapter 1", "Intro.json", "Untitled.json"]);
    }

    #[test]
    fn editor_renders_and_saves() {
        let (_tmp, mut editor) = editor();
        let folder = add(&mut editor, ROOT_ID, true);
        let id = add(&mut editor, &folder, false);
        let graph = editor.tree.graph_mut(&id).unwrap();
        let text = graph.add_node(NodeType::Text, pos2(0.0, 0.0));
        graph.node_mut(&text).unwrap().data = NodeData::Text(TextData {
            text: "Hello".into(),
            ..Default::default()
        });
        let container = graph.add_node(NodeType::BranchContainer, pos2(400.0, 0.0));
        graph.add_child(&container, NodeType::Branch, None);
        graph.add_child(&container, NodeType::SkillCheck, None);
        editor.open_folders.insert(folder);
        editor.dirty.insert(id.clone());
        let path = editor.path_of(&id).unwrap();

        let mut harness = Harness::builder()
            .with_size(vec2(1200.0, 800.0))
            .build_ui_state(
                |ui, e: &mut Editor| {
                    e.ui(ui, true);
                },
                editor,
            );
        harness.run();
        harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
        harness.run();

        let editor = harness.state();
        assert!(!editor.is_dirty(), "Ctrl+S should save");
        let on_disk = storage::parse_dialogue(&fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(on_disk.nodes, editor.tree.graph(&id).unwrap().nodes);
    }
}

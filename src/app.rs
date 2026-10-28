//! Top-level app state: the repository screen and the editor.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result, bail};
use eframe::egui::{
    self, Align, Frame, Id, Key, KeyboardShortcut, Layout, Modal, Modifiers, RichText, ScrollArea,
    Sense, Stroke, TextEdit, TextureHandle, TextureOptions, Ui, ViewportCommand, vec2,
};

use crate::database_modal::{DatabaseModal, EntityRequest, Fulfilled};
use crate::db::Database;
use crate::export::export_graph;
use crate::graph_editor::{GraphEnv, GraphEvent, GraphView};
use crate::images::{self, TextureCache};
use crate::model::{DialogueTree, Entities, NodeData, ROOT_ID, Repository, TreeItemKind};
use crate::repo_screen::RepoScreen;
use crate::repository::{self, OpenRepository};
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
                    editor.open_dialogue = editor
                        .tree
                        .preorder()
                        .into_iter()
                        .find(|i| !i.is_folder())
                        .map(|i| i.id.clone());
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
        let dirty = matches!(&self.screen, Screen::Editor(e) if e.dirty);
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
                let marker = if e.dirty { " •" } else { "" };
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
            && matches!(&self.screen, Screen::Editor(e) if e.dirty)
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
    Delete(String),
    Move(String, String),
    Add { parent: String, folder: bool },
}

pub struct Editor {
    repo: Repository,
    manifest_path: PathBuf,
    db: Database,
    tree: DialogueTree,
    entities: Entities,
    /// The highlighted sidebar item (folder or dialogue).
    selected: Option<String>,
    /// The dialogue shown on the canvas.
    open_dialogue: Option<String>,
    renaming: Option<(String, String)>,
    focus_rename: bool,
    open_folders: HashSet<String>,
    views: HashMap<String, GraphView>,
    db_modal: Option<DatabaseModal>,
    textures: TextureCache,
    dirty: bool,
    status: Option<String>,
    error: Option<String>,
}

impl Editor {
    fn load(open: OpenRepository) -> Result<Self> {
        let OpenRepository {
            repository,
            manifest_path,
            db,
        } = open;
        let tree = db.load_dialogues().context("failed to load dialogues")?;
        let entities = Entities {
            portraits: db.all_portraits()?,
            speakers: db.all_speakers()?,
            skill_categories: db.all_skill_categories()?,
        };
        Ok(Self {
            repo: repository,
            manifest_path,
            db,
            tree,
            entities,
            selected: None,
            open_dialogue: None,
            renaming: None,
            focus_rename: false,
            open_folders: HashSet::new(),
            views: HashMap::new(),
            db_modal: None,
            textures: TextureCache::default(),
            dirty: false,
            status: None,
            error: None,
        })
    }

    /// Writes all dialogues to the database. Returns whether it worked.
    fn save(&mut self) -> bool {
        match self.db.save_dialogues(&self.tree) {
            Ok(()) => {
                self.dirty = false;
                self.status = Some("Saved.".into());
                true
            }
            Err(err) => {
                self.error = Some(format!("{err:#}"));
                false
            }
        }
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
            .set_title("Save JSON")
            .set_file_name(format!("{}.json", item.name))
            .add_filter("JSON Files", &["json"])
            .save_file()
        else {
            return Ok(());
        };
        let json = serde_json::to_string_pretty(&export_graph(graph))?;
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
                    if ui.button("Add Dialogue").clicked() {
                        self.apply_tree_actions(vec![TreeAction::Add {
                            parent: ROOT_ID.into(),
                            folder: false,
                        }]);
                    }
                    let selected = self.selected.clone();
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Rename Dialogue"))
                        .clicked()
                        && let Some(id) = selected.clone()
                    {
                        self.apply_tree_actions(vec![TreeAction::StartRename(id)]);
                    }
                    if ui
                        .add_enabled(selected.is_some(), egui::Button::new("Remove Dialogue"))
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
                    let marker = if self.dirty { " (unsaved)" } else { "" };
                    ui.label(
                        RichText::new(format!("{}{marker}", self.repo.name))
                            .color(theme::SMALL_TEXT),
                    );
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
                &self.db,
                &mut self.entities,
                &self.repo,
                &mut self.textures,
            );
            if let Some(dir) = out.godot_project {
                self.repo.godot_project_location = Some(dir.to_string_lossy().into_owned());
                if let Err(err) = repository::save_manifest(&self.repo, &self.manifest_path) {
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

        action
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
            Fulfilled::Speaker(s) => data.speaker = Some(s),
            Fulfilled::Portrait(p) => data.portrait = Some(p),
        }
        if let Some(view) = self.views.get_mut(&request.dialogue_id) {
            view.show_text_field(&request.node_id, request.kind);
        }
        self.dirty = true;
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
                keyboard: interactive && self.db_modal.is_none() && self.renaming.is_none(),
            },
        );
        self.dirty |= out.changed;
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
            let resp = tree_row(ui, &name, indent, selected, is_folder.then_some(open));

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
                TreeAction::CommitRename => {
                    if let Some((id, name)) = self.renaming.take()
                        && let Some(item) = self.tree.get_mut(&id)
                        && !name.trim().is_empty()
                        && item.name != name.trim()
                    {
                        item.name = name.trim().to_string();
                        self.dirty = true;
                    }
                }
                TreeAction::CancelRename => self.renaming = None,
                TreeAction::Delete(id) => {
                    self.tree.remove(&id);
                    if self
                        .open_dialogue
                        .as_ref()
                        .is_some_and(|d| self.tree.get(d).is_none())
                    {
                        self.open_dialogue = None;
                    }
                    if self
                        .selected
                        .as_ref()
                        .is_some_and(|s| self.tree.get(s).is_none())
                    {
                        self.selected = None;
                    }
                    self.views.retain(|d, _| self.tree.get(d).is_some());
                    self.dirty = true;
                }
                TreeAction::Move(id, folder) => {
                    if self.tree.move_item(&id, &folder) {
                        self.open_folders.insert(folder);
                        self.dirty = true;
                    }
                }
                TreeAction::Add { parent, folder } => {
                    let kind = if folder {
                        TreeItemKind::Folder
                    } else {
                        TreeItemKind::Dialogue(Default::default())
                    };
                    let id = self.tree.add(&parent, kind);
                    if parent != ROOT_ID {
                        self.open_folders.insert(parent);
                    }
                    if !folder {
                        self.open_dialogue = Some(id.clone());
                    }
                    self.selected = Some(id.clone());
                    self.renaming = Some((id, "Untitled".into()));
                    self.focus_rename = true;
                    self.dirty = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Graph, NodeType, TextData};
    use eframe::egui::pos2;
    use egui_kittest::Harness;

    #[test]
    fn editor_renders_and_saves() {
        let tmp = tempfile::tempdir().unwrap();
        let open = repository::create("repo", tmp.path(), None).unwrap();
        let mut editor = Editor::load(open).unwrap();

        let folder = editor.tree.add(ROOT_ID, TreeItemKind::Folder);
        let id = editor
            .tree
            .add(&folder, TreeItemKind::Dialogue(Graph::default()));
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
        editor.open_dialogue = Some(id);
        editor.dirty = true;

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
        assert!(!editor.dirty, "Ctrl+S should save");
        assert_eq!(editor.db.load_dialogues().unwrap(), editor.tree);
    }
}

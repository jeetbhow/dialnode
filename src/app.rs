//! Top-level app state: the menu bar and the editor for the open folder.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use eframe::egui::{
    self, Align, Frame, Id, Key, KeyboardShortcut, Layout, Modal, Modifiers, RichText, ScrollArea,
    Sense, Stroke, TextEdit, TextureHandle, TextureOptions, Ui, ViewportCommand, vec2,
};

use crate::graph_editor::GraphView;
use crate::images;
use crate::model::{APP_VERSION, DialogueTree, Graph, ROOT_ID, TreeItemKind};
use crate::storage;
use crate::theme;

const SAVE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);

pub struct DialnodeApp {
    /// The open folder, if any.
    editor: Option<Box<Editor>>,
    logo: Option<TextureHandle>,
    confirm: Option<PendingExit>,
    allow_close: bool,
    title: String,
    /// Shown while no folder is open; the editor has its own.
    error: Option<String>,
    #[cfg(debug_assertions)]
    debug: crate::debug_hooks::DebugHooks,
}

/// Something that leaves the open folder, so it asks about unsaved changes
/// first.
#[derive(Clone, Copy)]
enum PendingExit {
    Quit,
    OpenFolder,
}

impl DialnodeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let logo = images::decode(include_bytes!("../assets/logo.png")).map(|img| {
            cc.egui_ctx
                .load_texture("logo", img, TextureOptions::LINEAR)
        });
        #[allow(unused_mut)]
        let mut editor = None;
        #[cfg(debug_assertions)]
        if let Some(path) = crate::debug_hooks::DebugHooks::folder_to_open() {
            match Editor::load(&path) {
                Ok(mut loaded) => {
                    loaded.open_dialogue = loaded.tree.dialogues().into_iter().next();
                    loaded.open_folders.extend(
                        loaded
                            .tree
                            .items
                            .iter()
                            .filter(|i| i.is_folder())
                            .map(|i| i.id.clone()),
                    );
                    editor = Some(Box::new(loaded));
                }
                Err(err) => eprintln!("DIALNODE_OPEN failed: {err:#}"),
            }
        }
        Self::with_editor(editor, logo)
    }

    fn with_editor(editor: Option<Box<Editor>>, logo: Option<TextureHandle>) -> Self {
        Self {
            editor,
            logo,
            confirm: None,
            allow_close: false,
            title: String::new(),
            error: None,
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
            PendingExit::OpenFolder => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Open Folder")
                    .pick_folder()
                {
                    self.open_editor(&path);
                }
            }
        }
    }

    /// Switches the editor to the folder `root`, or reports why it couldn't
    /// be opened.
    fn open_editor(&mut self, root: &Path) {
        match Editor::load(root) {
            Ok(editor) => {
                self.editor = Some(Box::new(editor));
                self.error = None;
            }
            Err(err) => {
                let message = format!("{err:#}");
                match &mut self.editor {
                    Some(editor) => editor.error = Some(message),
                    None => self.error = Some(message),
                }
            }
        }
    }

    fn request_exit(&mut self, ctx: &egui::Context, exit: PendingExit) {
        let dirty = self.editor.as_ref().is_some_and(|e| e.is_dirty());
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
                if theme::primary_button(ui, "Save").clicked() {
                    choice = Some(true);
                }
                if theme::secondary_button(ui, "Discard").clicked() {
                    choice = Some(false);
                }
                if ui.button("Cancel").clicked() {
                    self.confirm = None;
                }
            });
        });
        match choice {
            Some(true) => {
                if let Some(editor) = &mut self.editor
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
        let title = match &self.editor {
            None => "Dialnode".to_string(),
            Some(e) => {
                let marker = if e.is_dirty() { " •" } else { "" };
                format!("{}{marker} — Dialnode", e.name)
            }
        };
        if title != self.title {
            ctx.send_viewport_cmd(ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn show(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        if ctx.input(|i| i.viewport().close_requested())
            && !self.allow_close
            && self.editor.as_ref().is_some_and(|e| e.is_dirty())
        {
            ctx.send_viewport_cmd(ViewportCommand::CancelClose);
            self.confirm = Some(PendingExit::Quit);
        }

        if let Some(exit) = self.menu_bar(ui) {
            self.request_exit(&ctx, exit);
        }
        let interactive = self.confirm.is_none();
        match &mut self.editor {
            Some(editor) => editor.ui(ui, interactive),
            None => {
                if let Some(exit) = self.empty_editor(ui) {
                    self.proceed(&ctx, exit);
                }
            }
        }

        self.confirm_dialog(&ctx);
        self.update_title(&ctx);
    }

    fn menu_bar(&mut self, ui: &mut Ui) -> Option<PendingExit> {
        let mut exit = None;
        egui::Panel::top("menu-bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Folder…").clicked() {
                        exit = Some(PendingExit::OpenFolder);
                    }
                    if let Some(editor) = &mut self.editor {
                        ui.separator();
                        editor.file_menu(ui);
                    }
                    ui.separator();
                    if ui.button("Exit").clicked() {
                        exit = Some(PendingExit::Quit);
                    }
                });
                if let Some(editor) = &mut self.editor {
                    editor.menus(ui);
                }
            });
        });
        exit
    }

    /// The editor before a folder is open.
    fn empty_editor(&mut self, ui: &mut Ui) -> Option<PendingExit> {
        let mut exit = None;
        status_bar(ui, &mut self.error, None);
        sidebar_panel(ui).show(ui, |ui| {
            ui.add_space(6.0);
            ui.label(RichText::new("No folder open.").color(theme::SMALL_TEXT));
        });
        egui::CentralPanel::default().show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space((ui.available_height() * 0.2).max(24.0));
                if let Some(logo) = &self.logo {
                    ui.add(
                        egui::Image::new(logo)
                            .max_size(vec2(250.0, 200.0))
                            .corner_radius(14),
                    );
                }
                ui.label(
                    RichText::new(format!("Version: {APP_VERSION}"))
                        .small()
                        .color(theme::SMALL_TEXT),
                );
                ui.add_space(24.0);
                ui.label("Open a folder of dialogues, or an empty folder to start a new one.");
                ui.add_space(12.0);
                if theme::primary_button(ui, "Open Folder…").clicked() {
                    exit = Some(PendingExit::OpenFolder);
                }
            });
        });
        exit
    }
}

impl eframe::App for DialnodeApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
        #[cfg(debug_assertions)]
        self.debug.after_frame(ui.ctx());
    }
}

fn sidebar_panel(ui: &Ui) -> egui::Panel {
    egui::Panel::left("sidebar")
        .resizable(true)
        .default_size(300.0)
        .min_size(220.0)
        .frame(Frame::side_top_panel(ui.style()).fill(theme::SIDEBAR_BG))
}

/// Shows `error` with a button to dismiss it, or else `status`.
fn status_bar(ui: &mut Ui, error: &mut Option<String>, status: Option<&str>) {
    egui::Panel::bottom("status-bar").show(ui, |ui| {
        ui.horizontal(|ui| {
            if let Some(err) = error.as_deref() {
                ui.colored_label(theme::ERROR, err);
                if ui.small_button("Dismiss").clicked() {
                    *error = None;
                }
            } else if let Some(status) = status {
                ui.label(RichText::new(status).color(theme::SMALL_TEXT));
            }
        });
    });
}

/// One sidebar row. `folder_open` is `Some` for folders and draws a
/// disclosure triangle.
fn tree_row(
    ui: &mut Ui,
    name: &str,
    indent: f32,
    selected: bool,
    folder_open: Option<bool>,
    dragging: bool,
    drop_allowed: Option<bool>,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_exact_size(vec2(ui.available_width(), 24.0), Sense::click_and_drag());
    if !ui.is_rect_visible(rect) {
        return resp;
    }
    let painter = ui.painter();
    let drop_hovered = drop_allowed.is_some() && resp.contains_pointer();
    if drop_hovered {
        let color = if drop_allowed == Some(true) {
            theme::PRIMARY
        } else {
            theme::ERROR
        };
        painter.rect_filled(rect, 4.0, color.gamma_multiply(0.12));
        painter.rect_stroke(rect, 4.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    } else if dragging {
        painter.rect_filled(rect, 4.0, ui.visuals().widgets.inactive.weak_bg_fill);
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, theme::PRIMARY.gamma_multiply(0.4)),
            egui::StrokeKind::Inside,
        );
    } else if selected {
        painter.rect_filled(rect, 4.0, ui.visuals().selection.bg_fill);
    } else if resp.hovered() {
        painter.rect_filled(rect, 4.0, ui.visuals().widgets.hovered.weak_bg_fill);
    }
    let text_color = if dragging {
        theme::SMALL_TEXT
    } else {
        ui.visuals().text_color()
    };
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
    resp.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::accesskit::Role::Button,
            ui.is_enabled(),
            selected,
            name,
        )
    });
    resp
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

/// A dialogue in the open folder, with its editor state.
#[derive(Default)]
struct Dialogue {
    graph: Graph,
    view: GraphView,
    /// Changed since it was last written to its file.
    dirty: bool,
}

pub struct Editor {
    /// The folder's name, shown in the title and menu bar.
    name: String,
    /// The open folder.
    root: PathBuf,
    tree: DialogueTree<Dialogue>,
    /// The highlighted sidebar item (folder or dialogue).
    selected: Option<String>,
    /// The dialogue shown on the canvas.
    open_dialogue: Option<String>,
    renaming: Option<(String, String)>,
    focus_rename: bool,
    confirm_delete: Option<String>,
    open_folders: HashSet<String>,
    status: Option<String>,
    error: Option<String>,
}

impl Editor {
    /// Opens any folder: its dialogue files and subfolders make up the tree.
    fn load(root: &Path) -> Result<Self> {
        let (tree, warnings) = storage::load_tree(root).context("failed to read dialogues")?;
        let tree = tree.map(|graph| Dialogue {
            graph,
            ..Dialogue::default()
        });
        // A drive root has no name of its own.
        let name = root.file_name().map_or_else(
            || root.display().to_string(),
            |n| n.to_string_lossy().into_owned(),
        );
        let error = (!warnings.is_empty()).then(|| {
            format!(
                "Skipped {} file(s) that aren't valid dialogues: {}",
                warnings.len(),
                warnings.join("; ")
            )
        });
        Ok(Self {
            name,
            root: root.to_path_buf(),
            tree,
            selected: None,
            open_dialogue: None,
            renaming: None,
            focus_rename: false,
            confirm_delete: None,
            open_folders: HashSet::new(),
            status: None,
            error,
        })
    }

    /// Ids of the dialogues with unsaved changes.
    fn dirty_dialogues(&self) -> impl Iterator<Item = &str> {
        self.tree.items.iter().filter_map(|item| match &item.kind {
            TreeItemKind::Dialogue(d) if d.dirty => Some(item.id.as_str()),
            _ => None,
        })
    }

    fn is_dirty(&self) -> bool {
        self.dirty_dialogues().next().is_some()
    }

    /// Writes every changed dialogue to its file. Returns whether all of
    /// them were written.
    fn save(&mut self) -> bool {
        let ids: Vec<String> = self.dirty_dialogues().map(String::from).collect();
        for id in ids {
            let Some(path) = storage::path_of(&self.root, &self.tree, &id) else {
                continue;
            };
            let dialogue = self.tree.dialogue_mut(&id).expect("listed above");
            if let Err(err) = storage::write_dialogue(&path, &dialogue.graph) {
                self.error = Some(format!("{err:#}"));
                return false;
            }
            dialogue.dirty = false;
        }
        self.status = Some("Saved.".into());
        true
    }

    /// The File menu items that need an open folder.
    fn file_menu(&mut self, ui: &mut Ui) {
        let save =
            egui::Button::new("Save").shortcut_text(ui.ctx().format_shortcut(&SAVE_SHORTCUT));
        if ui.add(save).clicked() {
            self.save();
        }
    }

    /// The menu bar after the File menu: the Edit menu and the folder name.
    fn menus(&mut self, ui: &mut Ui) {
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
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let marker = if self.is_dirty() { " (unsaved)" } else { "" };
            ui.label(RichText::new(format!("{}{marker}", self.name)).color(theme::SMALL_TEXT))
                .on_hover_text(self.root.display().to_string());
        });
    }

    fn ui(&mut self, ui: &mut Ui, interactive: bool) {
        let ctx = ui.ctx().clone();

        if interactive && ctx.input_mut(|i| i.consume_shortcut(&SAVE_SHORTCUT)) {
            self.save();
        }

        status_bar(ui, &mut self.error, self.status.as_deref());
        sidebar_panel(ui).show(ui, |ui| self.sidebar(ui));
        self.inspector(ui);

        egui::CentralPanel::no_frame().show(ui, |ui| self.canvas(ui, interactive));
        self.delete_dialog(&ctx);
    }

    /// The right-hand panel that edits the selected node of the open
    /// dialogue. It only shows while a node is selected.
    fn inspector(&mut self, ui: &mut Ui) {
        let Some(id) = self.open_dialogue.clone() else {
            return;
        };
        let Some(Dialogue { graph, view, dirty }) = self.tree.dialogue_mut(&id) else {
            return;
        };
        if !view.has_selection(graph) {
            return;
        }
        let changed = egui::Panel::right("inspector")
            .resizable(true)
            .default_size(300.0)
            .min_size(240.0)
            .frame(Frame::side_top_panel(ui.style()).fill(theme::SIDEBAR_BG))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| view.inspector(ui, graph))
                    .inner
            })
            .inner;
        if changed {
            *dirty = true;
        }
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
                confirmed = theme::primary_button(ui, "Move to Trash").clicked();
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
        let keyboard = interactive && self.renaming.is_none() && self.confirm_delete.is_none();
        let Some(Dialogue { graph, view, dirty }) = self.tree.dialogue_mut(&id) else {
            self.open_dialogue = None;
            return;
        };
        if view.show(ui, graph, keyboard).changed {
            *dirty = true;
        }
    }

    fn sidebar(&mut self, ui: &mut Ui) {
        let mut actions = Vec::new();
        let mut drop_allowed = false;
        let target_folder = self
            .selected
            .as_deref()
            .filter(|id| self.tree.get(id).is_some_and(|i| i.is_folder()))
            .unwrap_or(ROOT_ID)
            .to_string();

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if theme::new_item_button(ui, theme::Icon::Dialogue)
                .on_hover_text("New dialogue")
                .clicked()
            {
                actions.push(TreeAction::Add {
                    parent: target_folder.clone(),
                    folder: false,
                });
            }
            if theme::new_item_button(ui, theme::Icon::Folder)
                .on_hover_text("New folder")
                .clicked()
            {
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
                self.tree_level(ui, ROOT_ID, 0, &mut actions, &mut drop_allowed);
                // Dropping on the empty space below the list moves to the top level.
                let rest = ui.available_size().max(vec2(ui.available_width(), 40.0));
                let resp = ui.allocate_response(rest, Sense::hover());
                if let Some(dragged) = egui::DragAndDrop::payload::<String>(ui.ctx()) {
                    if resp.contains_pointer() {
                        drop_allowed = self.can_drop(&dragged, ROOT_ID);
                    }
                }
                if let Some(id) = resp.dnd_release_payload::<String>() {
                    if self.can_drop(&id, ROOT_ID) {
                        actions.push(TreeAction::Move((*id).clone(), ROOT_ID.into()));
                    }
                }
            });

        self.apply_tree_actions(actions);
        if egui::DragAndDrop::has_payload_of_type::<String>(ui.ctx()) {
            ui.ctx().set_cursor_icon(if drop_allowed {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::NoDrop
            });
        }
    }

    /// Use the same checks for the drop cue and the eventual move.
    fn can_drop(&self, id: &str, folder: &str) -> bool {
        self.tree.get(id).is_some_and(|item| {
            item.parent != folder
                && self.tree.can_move(id, folder)
                && !self.tree.name_taken(folder, &item.file_name(), Some(id))
        })
    }

    fn tree_level(
        &mut self,
        ui: &mut Ui,
        parent: &str,
        depth: usize,
        actions: &mut Vec<TreeAction>,
        drop_allowed: &mut bool,
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
                    let resp = ui
                        .scope(|ui| {
                            theme::outline_fields(ui);
                            ui.add(TextEdit::singleline(buffer).desired_width(f32::INFINITY))
                        })
                        .inner;
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
            let label = if self.tree.dialogue(&id).is_some_and(|d| d.dirty) {
                format!("{name} •")
            } else {
                name
            };
            let dragged = egui::DragAndDrop::payload::<String>(ui.ctx());
            let can_drop = dragged
                .as_ref()
                .is_some_and(|dragged| self.can_drop(dragged, &id));
            let resp = tree_row(
                ui,
                &label,
                indent,
                selected,
                is_folder.then_some(open),
                dragged.as_deref().is_some_and(|dragged| dragged == &id),
                (is_folder && dragged.is_some()).then_some(can_drop),
            );

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
                    *drop_allowed = can_drop;
                }
                if let Some(dragged) = resp.dnd_release_payload::<String>() {
                    if self.can_drop(&dragged, &id) {
                        actions.push(TreeAction::Move((*dragged).clone(), id.clone()));
                    }
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
                self.tree_level(ui, &id, depth + 1, actions, drop_allowed);
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
                if self.tree.dialogue(&id).is_some() {
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
                let name = storage::validate_name(&input, folder)?;
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
                    TreeItemKind::Dialogue(Dialogue::default())
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

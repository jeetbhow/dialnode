//! The database window for portraits, speakers, and skills. It can also be
//! opened by a text node asking for a speaker or portrait.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use eframe::egui::{
    self, Align, Color32, Id, Layout, Modal, RichText, ScrollArea, TextEdit, Ui, Vec2,
    collapsing_header::CollapsingState, vec2,
};

use crate::db::Database;
use crate::graph_editor::EntityRequestKind;
use crate::images::{self, IMAGE_EXTENSIONS, TextureCache};
use crate::model::{Entities, Portrait, Repository, Skill, SkillCategory, Speaker, new_id};
use crate::repository;
use crate::theme;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Portrait,
    Speaker,
    Skill,
}

pub struct EntityRequest {
    pub dialogue_id: String,
    pub node_id: String,
    pub kind: EntityRequestKind,
}

pub enum Fulfilled {
    Speaker(Speaker),
    Portrait(Portrait),
}

#[derive(Default)]
pub struct ModalOutput {
    pub close: bool,
    pub fulfilled: Option<(EntityRequest, Fulfilled)>,
    pub godot_project: Option<PathBuf>,
}

struct PendingImage {
    path: PathBuf,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

pub struct DatabaseModal {
    tab: Tab,
    request: Option<EntityRequest>,
    selected: Option<String>,
    new_speaker: String,
    new_portrait: String,
    pending_image: Option<PendingImage>,
    new_category: Option<String>,
    new_skill: HashMap<String, String>,
    error: Option<String>,
}

impl DatabaseModal {
    pub fn new(request: Option<EntityRequest>) -> Self {
        let tab = match request.as_ref().map(|r| r.kind) {
            Some(EntityRequestKind::Speaker) => Tab::Speaker,
            _ => Tab::Portrait,
        };
        Self {
            tab,
            request,
            selected: None,
            new_speaker: String::new(),
            new_portrait: String::new(),
            pending_image: None,
            new_category: None,
            new_skill: HashMap::new(),
            error: None,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        db: &Database,
        entities: &mut Entities,
        repo: &Repository,
        textures: &mut TextureCache,
    ) -> ModalOutput {
        let mut out = ModalOutput::default();
        let modal = Modal::new(Id::unique("database-modal")).show(ctx, |ui| {
            let size = (ctx.content_rect().size() * vec2(0.55, 0.65)).max(vec2(560.0, 420.0));
            ui.set_width(size.x);
            ui.set_min_height(size.y);

            ui.horizontal(|ui| {
                for (tab, label, kind) in [
                    (Tab::Portrait, "Portrait", Some(EntityRequestKind::Portrait)),
                    (Tab::Speaker, "Speaker", Some(EntityRequestKind::Speaker)),
                    (Tab::Skill, "Skill", None),
                ] {
                    let enabled = self.request.as_ref().is_none_or(|r| Some(r.kind) == kind);
                    let text = RichText::new(label).size(16.0);
                    if ui
                        .add_enabled(enabled, egui::Button::selectable(self.tab == tab, text))
                        .clicked()
                    {
                        self.tab = tab;
                        self.selected = None;
                    }
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(theme::icon_button("×"))
                        .on_hover_text("Close")
                        .clicked()
                    {
                        out.close = true;
                    }
                });
            });

            ui.horizontal(|ui| {
                let project = repo
                    .godot_project_location
                    .as_deref()
                    .unwrap_or("(no Godot project)");
                ui.label(
                    RichText::new(format!("Project: {project}"))
                        .small()
                        .color(theme::SMALL_TEXT),
                );
                if ui.small_button("Change…").clicked() {
                    match pick_godot_project() {
                        Ok(Some(dir)) => out.godot_project = Some(dir),
                        Ok(None) => {}
                        Err(err) => self.error = Some(err.to_string()),
                    }
                }
            });
            ui.separator();

            if let Some(err) = &self.error {
                ui.colored_label(theme::ERROR, err);
            }

            let result = match self.tab {
                Tab::Portrait => self.portraits_tab(ui, db, entities, repo, textures, &mut out),
                Tab::Speaker => self.speakers_tab(ui, db, entities, &mut out),
                Tab::Skill => self.skills_tab(ui, db, entities),
            };
            if let Err(err) = result {
                self.error = Some(format!("{err:#}"));
            }
        });
        if modal.should_close() {
            out.close = true;
        }
        out
    }

    fn portraits_tab(
        &mut self,
        ui: &mut Ui,
        db: &Database,
        entities: &mut Entities,
        repo: &Repository,
        textures: &mut TextureCache,
        out: &mut ModalOutput,
    ) -> Result<()> {
        let list_height = (ui.available_height() - 110.0).max(160.0);
        ui.horizontal_top(|ui| -> Result<()> {
            ui.allocate_ui(vec2(ui.available_width() * 0.5, list_height), |ui| {
                self.entity_list(ui, entities.portraits.iter().map(|p| (&p.id, &p.name)))
            })
            .inner
            .map(|id| -> Result<()> {
                db.delete_portrait(&id)?;
                entities.portraits.retain(|p| p.id != id);
                textures.forget(&id);
                Ok(())
            })
            .transpose()?;

            ui.vertical_centered(|ui| {
                let Some(p) = self.selected_portrait(entities) else {
                    return;
                };
                match textures.get(ui.ctx(), &p.id, &p.data_url) {
                    Some(texture) => {
                        ui.add(egui::Image::new(&texture).fit_to_exact_size(Vec2::splat(100.0)));
                    }
                    None => {
                        ui.label("(preview unavailable)");
                    }
                }
                ui.small(&p.filename);
                ui.small(format!("{} x {}", p.width, p.height));
                ui.small(&p.virtual_path);
            });
            Ok(())
        })
        .inner?;

        if self
            .request
            .as_ref()
            .is_some_and(|r| r.kind == EntityRequestKind::Portrait)
        {
            let selected = self.selected_portrait(entities).cloned();
            if ui
                .add_enabled(
                    selected.is_some(),
                    theme::secondary_button("Select Portrait"),
                )
                .clicked()
                && let (Some(portrait), Some(request)) = (selected, self.request.take())
            {
                out.fulfilled = Some((request, Fulfilled::Portrait(portrait)));
                out.close = true;
            }
        }

        ui.separator();
        ui.horizontal(|ui| -> Result<()> {
            let label = self
                .pending_image
                .as_ref()
                .and_then(|i| i.path.file_name())
                .map_or("Upload:".to_string(), |n| n.to_string_lossy().into_owned());
            ui.label(label);
            if ui.add(theme::secondary_button("Browse…")).clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("Images", IMAGE_EXTENSIONS)
                    .pick_file()
            {
                self.pending_image = Some(read_image(&path)?);
            }
            ui.add(TextEdit::singleline(&mut self.new_portrait).hint_text("Portrait Name"));
            if ui.add(theme::primary_button("Add Portrait")).clicked() {
                let portrait = self.build_portrait(repo)?;
                db.create_portrait(&portrait)?;
                entities.portraits.push(portrait);
                self.new_portrait.clear();
                self.pending_image = None;
                self.error = None;
            }
            Ok(())
        })
        .inner
    }

    fn build_portrait(&self, repo: &Repository) -> Result<Portrait> {
        let (Some(image), false) = (&self.pending_image, self.new_portrait.trim().is_empty())
        else {
            bail!("Choose an image and enter a name for the portrait.");
        };
        let Some(godot) = repo.godot_project_location.as_deref() else {
            bail!("You must set a godot project directory before you can add a portrait.");
        };
        let Ok(rel) = image.path.strip_prefix(godot) else {
            bail!("The image is not contained in the godot project directory.");
        };
        let rel_path = rel.to_string_lossy().into_owned();
        Ok(Portrait {
            id: new_id(),
            kind: "portrait".into(),
            name: self.new_portrait.trim().to_string(),
            data_url: images::to_data_url(&image.bytes, images::mime_type(&image.path)),
            width: image.width,
            height: image.height,
            path: image.path.to_string_lossy().into_owned(),
            virtual_path: format!("res://{}", rel_path.replace('\\', "/")),
            rel_path,
            filename: image
                .path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
        })
    }

    fn selected_portrait<'e>(&self, entities: &'e Entities) -> Option<&'e Portrait> {
        let id = self.selected.as_deref()?;
        entities.portraits.iter().find(|p| p.id == id)
    }

    fn speakers_tab(
        &mut self,
        ui: &mut Ui,
        db: &Database,
        entities: &mut Entities,
        out: &mut ModalOutput,
    ) -> Result<()> {
        let list_height = (ui.available_height() - 110.0).max(160.0);
        let deleted = ui
            .allocate_ui(vec2(ui.available_width(), list_height), |ui| {
                self.entity_list(ui, entities.speakers.iter().map(|s| (&s.id, &s.name)))
            })
            .inner;
        if let Some(id) = deleted {
            db.delete_speaker(&id)?;
            entities.speakers.retain(|s| s.id != id);
        }

        if self
            .request
            .as_ref()
            .is_some_and(|r| r.kind == EntityRequestKind::Speaker)
        {
            let selected = self
                .selected
                .as_deref()
                .and_then(|id| entities.speakers.iter().find(|s| s.id == id))
                .cloned();
            if ui
                .add_enabled(
                    selected.is_some(),
                    theme::secondary_button("Select Speaker"),
                )
                .clicked()
                && let (Some(speaker), Some(request)) = (selected, self.request.take())
            {
                out.fulfilled = Some((request, Fulfilled::Speaker(speaker)));
                out.close = true;
            }
        }

        ui.separator();
        ui.horizontal(|ui| -> Result<()> {
            ui.add(TextEdit::singleline(&mut self.new_speaker).hint_text("Speaker Name"));
            let name = self.new_speaker.trim();
            if ui
                .add_enabled(!name.is_empty(), theme::primary_button("Add Speaker"))
                .clicked()
            {
                let speaker = Speaker::new(name.to_string());
                db.create_speaker(&speaker)?;
                entities.speakers.push(speaker);
                self.new_speaker.clear();
            }
            Ok(())
        })
        .inner
    }

    fn skills_tab(&mut self, ui: &mut Ui, db: &Database, entities: &mut Entities) -> Result<()> {
        let list_height = (ui.available_height() - 60.0).max(160.0);
        let mut delete_category = None;
        let mut delete_skill = None;
        let mut add_skill = None;

        ScrollArea::vertical()
            .max_height(list_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for category in &entities.skill_categories {
                    let id = Id::unique(("skill-category", &category.id));
                    CollapsingState::load_with_default_open(ui.ctx(), id, false)
                        .show_header(ui, |ui| {
                            ui.strong(&category.name);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if ui
                                    .add(theme::icon_button("×"))
                                    .on_hover_text("Delete category")
                                    .clicked()
                                {
                                    delete_category = Some(category.id.clone());
                                }
                            });
                        })
                        .body(|ui| {
                            for skill in &category.skills {
                                ui.horizontal(|ui| {
                                    ui.label(&skill.name);
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui
                                            .add(theme::icon_button("×"))
                                            .on_hover_text("Delete skill")
                                            .clicked()
                                        {
                                            delete_skill = Some(skill.id.clone());
                                        }
                                    });
                                });
                            }
                            add_skill = add_skill.take().or(self.add_skill_row(ui, category));
                        });
                }
                if entities.skill_categories.is_empty() {
                    ui.label(RichText::new("No skill categories yet.").color(theme::SMALL_TEXT));
                }
            });

        ui.separator();
        let mut add_category = None;
        ui.horizontal(|ui| match &mut self.new_category {
            Some(name) => {
                ui.add(TextEdit::singleline(name).hint_text("Skill Category Name"));
                if ui.add(theme::secondary_button("Add")).clicked() && !name.trim().is_empty() {
                    add_category = Some(name.trim().to_string());
                }
                if ui.add(theme::secondary_button("Cancel")).clicked() {
                    self.new_category = None;
                }
            }
            None => {
                if ui
                    .add(theme::primary_button("+ Add Skill Category"))
                    .clicked()
                {
                    self.new_category = Some(String::new());
                }
            }
        });

        if let Some(name) = add_category {
            let id = new_id();
            db.create_skill_category(&id, &name)?;
            entities.skill_categories.push(SkillCategory {
                id,
                name,
                skills: Vec::new(),
            });
            self.new_category = None;
        }
        if let Some(skill) = add_skill {
            db.create_skill(&skill)?;
            if let Some(c) = entities
                .skill_categories
                .iter_mut()
                .find(|c| c.id == skill.category_id)
            {
                c.skills.push(skill);
            }
        }
        if let Some(id) = delete_skill {
            db.delete_skill(&id)?;
            for c in &mut entities.skill_categories {
                c.skills.retain(|s| s.id != id);
            }
        }
        if let Some(id) = delete_category {
            db.delete_skill_category(&id)?;
            entities.skill_categories.retain(|c| c.id != id);
        }
        Ok(())
    }

    /// The "add skill" controls under a category. Returns a skill to create.
    fn add_skill_row(&mut self, ui: &mut Ui, category: &SkillCategory) -> Option<Skill> {
        let mut result = None;
        ui.horizontal(|ui| match self.new_skill.get_mut(&category.id) {
            Some(name) => {
                ui.add(TextEdit::singleline(name).hint_text("Skill Name"));
                if ui.small_button("Add").clicked() && !name.trim().is_empty() {
                    result = Some(Skill::new(category.id.clone(), name.trim().to_string()));
                    name.clear();
                }
                if ui.small_button("Cancel").clicked() {
                    self.new_skill.remove(&category.id);
                }
            }
            None => {
                if ui.small_button("+ Add Skill").clicked() {
                    self.new_skill.insert(category.id.clone(), String::new());
                }
            }
        });
        result
    }

    /// A selectable list with delete buttons. Returns the id to delete.
    fn entity_list<'a>(
        &mut self,
        ui: &mut Ui,
        items: impl Iterator<Item = (&'a String, &'a String)>,
    ) -> Option<String> {
        let mut deleted = None;
        ScrollArea::vertical()
            .id_salt("entity-list")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut any = false;
                for (id, name) in items {
                    any = true;
                    ui.horizontal(|ui| {
                        let selected = self.selected.as_ref() == Some(id);
                        let width = ui.available_width() - 32.0;
                        if ui
                            .add_sized(
                                [width, 24.0],
                                egui::Button::selectable(selected, name.as_str()),
                            )
                            .clicked()
                        {
                            self.selected = Some(id.clone());
                        }
                        if ui
                            .add(theme::icon_button("×"))
                            .on_hover_text("Delete")
                            .clicked()
                        {
                            deleted = Some(id.clone());
                        }
                    });
                }
                if !any {
                    ui.label(RichText::new("Nothing here yet.").color(Color32::GRAY));
                }
            });
        if deleted.is_some() && deleted == self.selected {
            self.selected = None;
        }
        deleted
    }
}

fn read_image(path: &Path) -> Result<PendingImage> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    let (width, height) = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()?
        .into_dimensions()
        .context("unsupported image")?;
    Ok(PendingImage {
        path: path.to_path_buf(),
        bytes,
        width,
        height,
    })
}

/// Asks for a folder and checks that it is a Godot project.
pub fn pick_godot_project() -> Result<Option<PathBuf>> {
    let Some(dir) = rfd::FileDialog::new()
        .set_title("Godot Project Location")
        .pick_folder()
    else {
        return Ok(None);
    };
    if !repository::is_godot_project(&dir) {
        bail!("Failed to locate the project.godot file at the root of the directory.");
    }
    Ok(Some(dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Speaker;
    use egui_kittest::Harness;

    #[test]
    fn every_tab_renders() {
        let db = Database::open_in_memory().unwrap();
        let mut entities = Entities::default();
        entities.speakers.push(Speaker::new("Jeet".into()));
        entities.skill_categories.push(SkillCategory {
            id: "c".into(),
            name: "Body".into(),
            skills: vec![Skill::new("c".into(), "Strength".into())],
        });
        let repo = Repository {
            id: "r".into(),
            name: "repo".into(),
            location: "/tmp".into(),
            godot_project_location: None,
            created_on: String::new(),
            app_version: String::new(),
            format_version: String::new(),
        };
        for tab in [Tab::Portrait, Tab::Speaker, Tab::Skill] {
            let mut modal = DatabaseModal::new(None);
            modal.tab = tab;
            modal.new_category = Some(String::new());
            modal.new_skill.insert("c".into(), String::new());
            let mut textures = TextureCache::default();
            let mut harness = Harness::new_ui(|ui| {
                CollapsingState::load_with_default_open(
                    ui.ctx(),
                    Id::unique(("skill-category", "c")),
                    true,
                )
                .store(ui.ctx());
                modal.show(ui.ctx(), &db, &mut entities, &repo, &mut textures);
            });
            harness.run();
        }
    }
}

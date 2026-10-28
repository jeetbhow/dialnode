//! The start screen: create or open a repository.

use std::path::PathBuf;

use anyhow::Result;
use eframe::egui::{
    self, Align, Label, Layout, Rect, RichText, TextEdit, TextureHandle, Ui, UiBuilder, vec2,
};

use crate::database_modal::pick_godot_project;
use crate::model::APP_VERSION;
use crate::repository::{self, DEFAULT_REPOSITORY_NAME, OpenRepository};
use crate::theme;

const COLUMN_WIDTH: f32 = 560.0;

#[derive(PartialEq)]
enum View {
    Main,
    Create,
}

pub struct RepoScreen {
    view: View,
    name: String,
    godot_location: Option<PathBuf>,
    custom_location: Option<PathBuf>,
    use_custom_location: bool,
    pub error: Option<String>,
}

impl Default for RepoScreen {
    fn default() -> Self {
        Self {
            view: View::Main,
            name: DEFAULT_REPOSITORY_NAME.into(),
            godot_location: None,
            custom_location: None,
            use_custom_location: false,
            error: None,
        }
    }
}

impl RepoScreen {
    pub fn ui(&mut self, ui: &mut Ui, logo: Option<&TextureHandle>) -> Option<OpenRepository> {
        let mut opened = None;
        egui::CentralPanel::default().show(ui, |ui| {
            let full = ui.max_rect();
            let column = Rect::from_min_max(
                egui::pos2(full.center().x - COLUMN_WIDTH / 2.0, full.top() + 24.0),
                egui::pos2(full.center().x + COLUMN_WIDTH / 2.0, full.bottom()),
            );
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(column)
                    .layout(Layout::top_down(Align::Center)),
                |ui| {
                    if let Some(logo) = logo {
                        ui.add(egui::Image::new(logo).max_size(vec2(250.0, 200.0)));
                    }
                    ui.label(
                        RichText::new(format!("Version: {APP_VERSION}"))
                            .small()
                            .color(theme::SMALL_TEXT),
                    );
                    ui.add_space(32.0);

                    if let Some(err) = &self.error {
                        ui.colored_label(theme::ERROR, err);
                        ui.add_space(8.0);
                    }

                    ui.with_layout(Layout::top_down(Align::Min), |ui| {
                        let result = match self.view {
                            View::Main => self.main_view(ui),
                            View::Create => self.create_view(ui),
                        };
                        match result {
                            Ok(repo) => opened = repo,
                            Err(err) => self.error = Some(format!("{err:#}")),
                        }
                    });
                },
            );
        });
        opened
    }

    fn main_view(&mut self, ui: &mut Ui) -> Result<Option<OpenRepository>> {
        let mut create = false;
        let mut open = false;
        row(
            ui,
            "Create a new repository",
            "Create a new local Dialnode repository.",
            true,
            |ui, _| {
                create = ui.add(theme::primary_button("Create")).clicked();
            },
        );
        row(
            ui,
            "Open a repository",
            "Open an already existing local Dialnode repository.",
            true,
            |ui, _| {
                open = ui.add(theme::secondary_button("Open")).clicked();
            },
        );

        if create {
            self.view = View::Create;
            self.error = None;
        }
        if open
            && let Some(path) = rfd::FileDialog::new()
                .set_title("Open Repository")
                .add_filter("Dialnode (.dial)", &["dial"])
                .pick_file()
        {
            return repository::open(&path).map(Some);
        }
        Ok(None)
    }

    fn create_view(&mut self, ui: &mut Ui) -> Result<Option<OpenRepository>> {
        if ui.button("< Back").clicked() {
            *self = Self::default();
            return Ok(None);
        }
        ui.heading("Create a Repository");
        ui.separator();
        ui.add_space(16.0);

        row(
            ui,
            "Name",
            "Set a custom name for the repository.",
            true,
            |ui, title| {
                ui.add(TextEdit::singleline(&mut self.name).desired_width(180.0))
                    .labelled_by(title);
            },
        );

        let godot_desc = self.godot_location.as_ref().map_or(
            "Will also be the location of your repository unless you're using a custom one."
                .to_string(),
            |p| p.display().to_string(),
        );
        let mut browse_godot = false;
        row(
            ui,
            "Godot Project Location",
            &godot_desc,
            !self.use_custom_location,
            |ui, _| {
                browse_godot = ui
                    .add_enabled(!self.use_custom_location, theme::secondary_button("Browse"))
                    .clicked();
            },
        );
        if browse_godot && let Some(dir) = pick_godot_project()? {
            self.godot_location = Some(dir);
        }

        let custom_desc = self.custom_location.as_ref().map_or(
            "In case you don't have a Godot project. Some features will be inaccessible."
                .to_string(),
            |p| p.display().to_string(),
        );
        let mut browse_custom = false;
        let mut toggled = false;
        row(ui, "Custom Location", &custom_desc, true, |ui, _| {
            browse_custom = ui
                .add_enabled(self.use_custom_location, theme::secondary_button("Browse"))
                .clicked();
            toggled = ui.checkbox(&mut self.use_custom_location, "Use").changed();
        });
        if toggled {
            self.godot_location = None;
            self.custom_location = None;
        }
        if browse_custom
            && let Some(dir) = rfd::FileDialog::new()
                .set_title("Custom Repository")
                .pick_folder()
        {
            self.custom_location = Some(dir);
        }

        ui.add_space(16.0);
        let location = self.godot_location.clone().or(self.custom_location.clone());
        let clicked = ui
            .vertical_centered(|ui| {
                ui.add_enabled(location.is_some(), theme::primary_button("Create"))
                    .clicked()
            })
            .inner;
        if clicked && let Some(location) = location {
            return repository::create(&self.name, &location, self.godot_location.as_deref())
                .map(Some);
        }
        Ok(None)
    }
}

/// A settings-style row: title and description on the left, a control on the
/// right. The control gets the title's id so it can be labelled by it.
fn row(
    ui: &mut Ui,
    title: &str,
    description: &str,
    enabled: bool,
    control: impl FnOnce(&mut Ui, egui::Id),
) {
    ui.horizontal(|ui| {
        let text_width = (ui.available_width() - 200.0).max(200.0);
        let title_id = ui
            .allocate_ui(vec2(text_width, 0.0), |ui| {
                ui.vertical(|ui| {
                    let title =
                        ui.add_enabled(enabled, Label::new(RichText::new(title).size(16.0)));
                    ui.add_enabled(
                        enabled,
                        Label::new(RichText::new(description).small().color(theme::SMALL_TEXT))
                            .wrap(),
                    );
                    title.id
                })
                .inner
            })
            .inner;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            control(ui, title_id)
        });
    });
    ui.separator();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_views_render() {
        for view in [View::Main, View::Create] {
            let mut screen = RepoScreen {
                view,
                error: Some("example error".into()),
                ..Default::default()
            };
            let mut harness = egui_kittest::Harness::new_ui(|ui| {
                screen.ui(ui, None);
            });
            harness.run();
        }
    }
}

mod app;
mod database_modal;
mod db;
#[cfg(debug_assertions)]
mod debug_hooks;
mod export;
mod graph_editor;
mod images;
mod model;
mod repo_screen;
mod repository;
mod theme;

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

fn main() -> eframe::Result {
    #[cfg(target_os = "linux")]
    use_software_rendering_on_wsl();

    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("bundled icon is a valid PNG");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Dialnode")
            .with_app_id("dialnode")
            .with_inner_size([1200.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Dialnode",
        options,
        Box::new(|cc| {
            install_fonts(&cc.egui_ctx);
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::DialnodeApp::new(cc)))
        }),
    )
}

/// WSLg's GPU path usually fails to create a surface, so fall back to Mesa's
/// software renderer unless the user has configured Mesa themselves.
#[cfg(target_os = "linux")]
fn use_software_rendering_on_wsl() {
    let on_wsl = std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|release| release.to_lowercase().contains("microsoft"));
    let mesa_configured = std::env::vars_os().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("MESA_") || key == "LIBGL_ALWAYS_SOFTWARE" || key == "GALLIUM_DRIVER"
    });
    if on_wsl && !mesa_configured {
        // SAFETY: called at the top of main, before any other threads exist.
        unsafe { std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1") };
    }
}

/// Iosevka is the text-field font from the original app.
fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "iosevka".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/Iosevka-Regular.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "iosevka".into());
    ctx.set_fonts(fonts);
}

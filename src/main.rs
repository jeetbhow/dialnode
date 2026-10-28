mod app;
#[cfg(debug_assertions)]
mod debug_hooks;
mod graph_editor;
mod images;
mod model;
mod storage;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    #[cfg(target_os = "linux")]
    configure_wsl_graphics();

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
            theme::apply(&cc.egui_ctx);
            Ok(Box::new(app::DialnodeApp::new(cc)))
        }),
    )
}

/// Works around WSLg quirks:
/// - Its GPU path usually fails to create a surface, so fall back to Mesa's
///   software renderer unless the user has configured Mesa themselves.
/// - Its Wayland compositor intermittently rejects new windows ("Connection
///   reset by peer"), while XWayland is reliable, so use X11 when it's
///   available. Set `DIALNODE_WAYLAND=1` to keep Wayland.
#[cfg(target_os = "linux")]
fn configure_wsl_graphics() {
    let on_wsl = std::env::var_os("WSL_DISTRO_NAME").is_some()
        || std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .is_ok_and(|release| release.to_lowercase().contains("microsoft"));
    if !on_wsl {
        return;
    }
    let mesa_configured = std::env::vars_os().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("MESA_") || key == "LIBGL_ALWAYS_SOFTWARE" || key == "GALLIUM_DRIVER"
    });
    let use_x11 =
        std::env::var_os("DISPLAY").is_some() && std::env::var_os("DIALNODE_WAYLAND").is_none();

    // SAFETY: called at the top of main, before any other threads exist.
    unsafe {
        if !mesa_configured {
            std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");
        }
        if use_x11 {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }
}

//! Debug-build helpers for checking the UI without clicking through it.
//!
//! - `DIALNODE_OPEN=<folder>` opens a folder of dialogues on startup.
//! - `DIALNODE_SCREENSHOT=<file.png>` saves a screenshot after a few frames,
//!   then quits.

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui::{self, ColorImage, ScreenshotCallback, ViewportCommand};

const SCREENSHOT_FRAME: u32 = 60;

pub struct DebugHooks {
    screenshot: Option<PathBuf>,
    frame: u32,
}

impl DebugHooks {
    pub fn from_env() -> Self {
        Self {
            screenshot: std::env::var_os("DIALNODE_SCREENSHOT").map(PathBuf::from),
            frame: 0,
        }
    }

    pub fn folder_to_open() -> Option<PathBuf> {
        std::env::var_os("DIALNODE_OPEN").map(PathBuf::from)
    }

    pub fn after_frame(&mut self, ctx: &egui::Context) {
        let Some(path) = &self.screenshot else {
            return;
        };
        self.frame += 1;
        ctx.request_repaint();
        if self.frame != SCREENSHOT_FRAME {
            return;
        }
        let path = path.clone();
        let ctx_clone = ctx.clone();
        ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(
            move |image: Arc<ColorImage>| {
                let rgba: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|c| c.to_srgba_unmultiplied())
                    .collect();
                let [w, h] = image.size;
                if let Err(err) =
                    image::save_buffer(&path, &rgba, w as u32, h as u32, image::ColorType::Rgba8)
                {
                    eprintln!("failed to save screenshot: {err}");
                }
                ctx_clone.send_viewport_cmd(ViewportCommand::Close);
            },
        )));
    }
}

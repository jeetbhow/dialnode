//! Portrait images, loaded from the Godot project and cached as textures.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp"];

pub fn decode(bytes: &[u8]) -> Option<ColorImage> {
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, image.as_raw()))
}

/// Turns `res://a/b.png` into a path inside the Godot project.
pub fn resolve_res_path(godot_project: &Path, virtual_path: &str) -> Option<PathBuf> {
    let rel = virtual_path.strip_prefix("res://")?;
    Some(godot_project.join(rel))
}

/// Lazily loaded textures keyed by `res://` path. Failures are cached as
/// `None` so missing files aren't retried every frame.
#[derive(Default)]
pub struct TextureCache {
    textures: HashMap<String, Option<TextureHandle>>,
}

impl TextureCache {
    pub fn get(
        &mut self,
        ctx: &egui::Context,
        godot_project: Option<&Path>,
        virtual_path: &str,
    ) -> Option<TextureHandle> {
        self.textures
            .entry(virtual_path.to_string())
            .or_insert_with(|| {
                let path = resolve_res_path(godot_project?, virtual_path)?;
                let image = decode(&std::fs::read(path).ok()?)?;
                Some(ctx.load_texture(virtual_path, image, TextureOptions::LINEAR))
            })
            .clone()
    }

    /// Drops everything, e.g. after the Godot project location changes.
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn res_paths() {
        assert_eq!(
            resolve_res_path(Path::new("/game"), "res://art/a.png").unwrap(),
            Path::new("/game/art/a.png")
        );
        assert!(resolve_res_path(Path::new("/game"), "art/a.png").is_none());
    }
}

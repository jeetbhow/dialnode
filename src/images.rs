//! Portrait images: data URLs (the stored format) and egui textures.

use std::collections::HashMap;
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

pub const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "gif", "bmp", "webp"];

pub fn mime_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("bmp") => "image/bmp",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

pub fn to_data_url(bytes: &[u8], mime: &str) -> String {
    format!("data:{mime};base64,{}", STANDARD.encode(bytes))
}

pub fn from_data_url(url: &str) -> Option<Vec<u8>> {
    let (_, payload) = url.split_once(";base64,")?;
    STANDARD.decode(payload).ok()
}

pub fn decode(bytes: &[u8]) -> Option<ColorImage> {
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, image.as_raw()))
}

/// Lazily decoded textures keyed by portrait id. Failed decodes are cached as
/// `None` so they aren't retried every frame.
#[derive(Default)]
pub struct TextureCache {
    textures: HashMap<String, Option<TextureHandle>>,
}

impl TextureCache {
    pub fn get(&mut self, ctx: &egui::Context, key: &str, data_url: &str) -> Option<TextureHandle> {
        self.textures
            .entry(key.to_string())
            .or_insert_with(|| {
                let image = decode(&from_data_url(data_url)?)?;
                Some(ctx.load_texture(format!("portrait-{key}"), image, TextureOptions::LINEAR))
            })
            .clone()
    }

    pub fn forget(&mut self, key: &str) {
        self.textures.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_url_round_trip() {
        let url = to_data_url(b"abc", "image/png");
        assert_eq!(url, "data:image/png;base64,YWJj");
        assert_eq!(from_data_url(&url).unwrap(), b"abc");
    }
}

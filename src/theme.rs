//! Colours and widget helpers carried over from the original stylesheet.

use eframe::egui::{self, Button, Color32, RichText, Stroke, Theme, vec2};

pub const PRIMARY: Color32 = Color32::from_rgb(0x2b, 0x40, 0x7b);
pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(0xe4, 0xe7, 0xf4);
pub const SECONDARY: Color32 = Color32::from_rgb(0xd0, 0xcd, 0xcd);
pub const SECONDARY_TEXT: Color32 = Color32::from_rgb(0x2f, 0x2e, 0x2e);
pub const SMALL_TEXT: Color32 = Color32::from_rgb(0x6f, 0x6f, 0x72);
pub const TARGET_HANDLE: Color32 = Color32::from_rgb(0xf6, 0x5c, 0x5c);
pub const SOURCE_HANDLE: Color32 = Color32::from_rgb(0x63, 0x6f, 0xed);
pub const NODE_BG: Color32 = Color32::WHITE;
pub const NODE_BORDER: Color32 = Color32::from_rgb(0xdd, 0xdd, 0xe0);
pub const CANVAS_BG: Color32 = Color32::from_rgb(0xfc, 0xfc, 0xfc);
pub const CANVAS_DOT: Color32 = Color32::from_rgb(0xc4, 0xc4, 0xcb);
pub const EDGE: Color32 = Color32::from_rgb(0xb1, 0xb1, 0xb7);
pub const EDGE_HOVER: Color32 = Color32::from_rgb(0x55, 0x55, 0x5e);
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(0xfa, 0xfa, 0xfa);
pub const ERROR: Color32 = Color32::from_rgb(0xc0, 0x39, 0x2b);

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(Theme::Light);
    ctx.style_mut_of(Theme::Light, |style| {
        style.visuals.selection.bg_fill = Color32::from_rgb(0xd5, 0xdb, 0xef);
        style.visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(8.0, 4.0);
    });
}

pub fn primary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).color(PRIMARY_TEXT).strong())
        .fill(PRIMARY)
        .corner_radius(4)
        .min_size(vec2(80.0, 28.0))
}

pub fn secondary_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).color(SECONDARY_TEXT).strong())
        .fill(SECONDARY)
        .corner_radius(4)
        .min_size(vec2(80.0, 28.0))
}

/// A frameless icon-style button, used for the delete crosses on nodes.
pub fn icon_button(text: &str) -> Button<'static> {
    Button::new(RichText::new(text).size(18.0)).frame(false)
}

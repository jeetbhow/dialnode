//! Colours and widget helpers carried over from the original stylesheet.

use eframe::egui::{
    self, Button, Color32, CornerRadius, Painter, Pos2, Response, RichText, Sense, Shape, Stroke,
    TextStyle, Theme, Ui, pos2, vec2,
};

/// Accent used for text, strokes and icons; light enough to read on dark panels.
pub const PRIMARY: Color32 = Color32::from_rgb(0x9a, 0xa8, 0xff);
/// Solid fill behind primary buttons.
pub const PRIMARY_FILL: Color32 = Color32::from_rgb(0x4f, 0x5f, 0xd6);
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0x60, 0x70, 0xe8);
pub const PRIMARY_ACTIVE: Color32 = Color32::from_rgb(0x42, 0x51, 0xbd);
pub const PRIMARY_TEXT: Color32 = Color32::from_rgb(0xf4, 0xf5, 0xff);
pub const SECONDARY: Color32 = Color32::from_rgb(0x23, 0x27, 0x36);
pub const SECONDARY_HOVER: Color32 = Color32::from_rgb(0x2d, 0x32, 0x46);
pub const SECONDARY_ACTIVE: Color32 = Color32::from_rgb(0x38, 0x3e, 0x58);
pub const SECONDARY_TEXT: Color32 = PRIMARY;
pub const TEXT: Color32 = Color32::from_rgb(0xd8, 0xdc, 0xe8);
pub const SMALL_TEXT: Color32 = Color32::from_rgb(0x8a, 0x90, 0xa3);
pub const NODE_BG: Color32 = Color32::from_rgb(0x1a, 0x1d, 0x27);
pub const NODE_BORDER: Color32 = Color32::from_rgb(0x2c, 0x31, 0x42);
pub const CANVAS_BG: Color32 = Color32::from_rgb(0x0f, 0x11, 0x17);
pub const CANVAS_GRID: Color32 = Color32::from_rgb(0x16, 0x19, 0x22);
pub const CANVAS_GRID_MAJOR: Color32 = Color32::from_rgb(0x1e, 0x22, 0x2e);
pub const EDGE: Color32 = Color32::from_rgb(0x4d, 0x54, 0x6b);
pub const EDGE_HOVER: Color32 = Color32::from_rgb(0xa5, 0xad, 0xc8);
pub const SIDEBAR_BG: Color32 = Color32::from_rgb(0x14, 0x16, 0x1e);
pub const OVERLAY_BG: Color32 = Color32::from_rgba_premultiplied(0x15, 0x17, 0x1f, 0xe6);
pub const MINIMAP_NODE: Color32 = Color32::from_rgb(0x3a, 0x40, 0x56);
pub const ERROR: Color32 = Color32::from_rgb(0xff, 0x6b, 0x5e);

// One accent per kind of node, shared by its toolbar button and its cards.
pub const NODE_START: Color32 = Color32::from_rgb(0x5f, 0xd6, 0xa4);
pub const NODE_TEXT: Color32 = Color32::from_rgb(0x8f, 0x9c, 0xff);
pub const NODE_BRANCH: Color32 = Color32::from_rgb(0xf2, 0xb4, 0x5a);
pub const NODE_SKILL: Color32 = Color32::from_rgb(0xc7, 0x8b, 0xff);
pub const NODE_END: Color32 = Color32::from_rgb(0xff, 0x7b, 0x8a);

pub fn apply(ctx: &egui::Context) {
    ctx.set_theme(Theme::Dark);
    ctx.style_mut_of(Theme::Dark, |style| {
        let visuals = &mut style.visuals;
        visuals.panel_fill = SIDEBAR_BG;
        visuals.window_fill = NODE_BG;
        visuals.window_stroke = Stroke::new(1.0, NODE_BORDER);
        visuals.extreme_bg_color = CANVAS_BG;
        visuals.faint_bg_color = SECONDARY;
        visuals.selection.bg_fill = Color32::from_rgb(0x2b, 0x35, 0x73);
        visuals.selection.stroke = Stroke::new(1.0, PRIMARY);
        visuals.widgets.noninteractive.fg_stroke.color = TEXT;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, NODE_BORDER);
        // egui's defaults (12.5pt body in a thin font) look soft and small.
        for (text_style, size) in [
            (TextStyle::Small, 12.0),
            (TextStyle::Body, 15.0),
            (TextStyle::Button, 15.0),
            (TextStyle::Monospace, 14.0),
            (TextStyle::Heading, 22.0),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style) {
                font.size = size;
            }
        }
        // Tooltips appear almost at once (egui waits half a second).
        style.interaction.tooltip_delay = 0.1;
        style.spacing.item_spacing = vec2(8.0, 6.0);
        style.spacing.button_padding = vec2(10.0, 5.0);

        // Flat, soft buttons: no outline, gentle tint on hover and press.
        let widgets = &mut style.visuals.widgets;
        for state in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            state.corner_radius = CornerRadius::same(6);
            state.bg_stroke = Stroke::NONE;
            state.expansion = 0.0;
            state.fg_stroke.color = PRIMARY;
        }
        widgets.inactive.weak_bg_fill = SECONDARY;
        widgets.hovered.weak_bg_fill = SECONDARY_HOVER;
        widgets.active.weak_bg_fill = SECONDARY_ACTIVE;
    });
}

pub fn primary_button(ui: &mut Ui, text: &str) -> Response {
    filled_button(
        ui,
        text,
        PRIMARY_FILL,
        PRIMARY_HOVER,
        PRIMARY_ACTIVE,
        PRIMARY_TEXT,
    )
}

pub fn secondary_button(ui: &mut Ui, text: &str) -> Response {
    filled_button(
        ui,
        text,
        SECONDARY,
        SECONDARY_HOVER,
        SECONDARY_ACTIVE,
        SECONDARY_TEXT,
    )
}

/// A filled button whose fill darkens on hover and press.
fn filled_button(
    ui: &mut Ui,
    text: &str,
    base: Color32,
    hover: Color32,
    active: Color32,
    text_color: Color32,
) -> Response {
    ui.scope(|ui| {
        let widgets = &mut ui.visuals_mut().widgets;
        widgets.inactive.weak_bg_fill = base;
        widgets.hovered.weak_bg_fill = hover;
        widgets.active.weak_bg_fill = active;
        ui.add(
            Button::new(RichText::new(text).color(text_color))
                .corner_radius(6)
                .min_size(vec2(88.0, 32.0)),
        )
    })
    .inner
}

#[derive(Clone, Copy)]
pub enum Icon {
    Dialogue,
    Folder,
}

/// A square, painted "new item" button: a page or folder with a plus badge.
/// Drawn with shapes so it doesn't depend on the font's glyph coverage.
pub fn new_item_button(ui: &mut Ui, icon: Icon) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);

        let stroke = Stroke::new(1.5, visuals.fg_stroke.color);
        let c = rect.center();
        let badge = match icon {
            Icon::Dialogue => {
                let (l, r, t, b) = (c.x - 6.0, c.x + 6.0, c.y - 8.0, c.y + 8.0);
                painter.add(Shape::closed_line(
                    vec![
                        pos2(l, t),
                        pos2(r - 4.0, t),
                        pos2(r, t + 4.0),
                        pos2(r, b),
                        pos2(l, b),
                    ],
                    stroke,
                ));
                painter.add(Shape::line(
                    vec![pos2(r - 4.0, t), pos2(r - 4.0, t + 4.0), pos2(r, t + 4.0)],
                    stroke,
                ));
                pos2(r, b)
            }
            Icon::Folder => {
                let (l, r, t, b) = (c.x - 9.0, c.x + 9.0, c.y - 6.0, c.y + 7.0);
                painter.add(Shape::closed_line(
                    vec![
                        pos2(l, t),
                        pos2(l + 5.0, t),
                        pos2(l + 7.5, t + 2.5),
                        pos2(r, t + 2.5),
                        pos2(r, b),
                        pos2(l, b),
                    ],
                    stroke,
                ));
                pos2(r, b)
            }
        };
        // Plus badge, cut out of the background so it reads over the outline.
        painter.circle_filled(badge, 5.5, visuals.weak_bg_fill);
        painter.line_segment([badge - vec2(3.0, 0.0), badge + vec2(3.0, 0.0)], stroke);
        painter.line_segment([badge - vec2(0.0, 3.0), badge + vec2(0.0, 3.0)], stroke);
    }
    response
}

/// Gives text fields in this scope a visible outline. Buttons stay flat, so
/// this is applied per field instead of in `apply`.
pub fn outline_fields(ui: &mut Ui) {
    let widgets = &mut ui.visuals_mut().widgets;
    widgets.inactive.bg_stroke = Stroke::new(1.0, NODE_BORDER);
    widgets.hovered.bg_stroke = Stroke::new(1.0, EDGE);
}

/// A square icon button with the flat, soft tint on hover and press. `draw`
/// paints the icon, given the painter and the centre of the tile.
pub fn icon_tile(ui: &mut Ui, draw: impl FnOnce(&Painter, Pos2)) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::click_and_drag());
    if ui.is_rect_visible(rect) {
        let visuals = *ui.style().interact(&response);
        let painter = ui.painter();
        painter.rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
        draw(painter, rect.center());
    }
    response
}

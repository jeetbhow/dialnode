//! The node-graph canvas: pan/zoom, node widgets, handles, and edges.

use std::collections::HashMap;

use eframe::egui::{
    self, Align2, Area, Color32, CornerRadius, DragValue, FontId, Frame, Id, Key, Label, LayerId,
    Order, PointerButton, Pos2, Rect, RichText, Sense, Shadow, Shape, Stroke, TextEdit, Ui,
    UiBuilder, Vec2, epaint::CubicBezierShape, pos2, vec2,
};

use crate::model::{
    BRANCH_CONTAINER_MIN_HEIGHT, BRANCH_CONTAINER_MIN_WIDTH, Graph, NodeData, NodeType,
};
use crate::theme;

const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_HIT: f32 = 10.0;
/// How far a handle sits outside its card edge, in world units.
const HANDLE_OFFSET: f32 = 9.0;
const CURVATURE: f32 = 0.4;
const ARROW_SIZE: f32 = 10.0;
const CONTAINER_HEADER: f32 = 36.0;
const CHILD_INSET: f32 = 12.0;
const CHILD_GAP: f32 = 8.0;
const NODE_MARGIN: f32 = 8.0;
const GRID_SPACING: f32 = 20.0;
/// Every this many grid lines, one is drawn a little stronger.
const GRID_MAJOR_EVERY: i64 = 5;
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 3.0;
/// Fitting the view never magnifies past this, so small graphs aren't blown up.
const MAX_FIT_ZOOM: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Top,
    Right,
    Bottom,
}

impl Side {
    fn outward(self) -> Vec2 {
        match self {
            Side::Left => vec2(-1.0, 0.0),
            Side::Top => vec2(0.0, -1.0),
            Side::Right => vec2(1.0, 0.0),
            Side::Bottom => vec2(0.0, 1.0),
        }
    }

    fn opposite(self) -> Self {
        match self {
            Side::Left => Side::Right,
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
        }
    }

    fn anchor(self, rect: Rect) -> Pos2 {
        match self {
            Side::Left => rect.left_center(),
            Side::Top => rect.center_top(),
            Side::Right => rect.right_center(),
            Side::Bottom => rect.center_bottom(),
        }
    }
}

struct HandleDef {
    /// Matches the handle ids the Electron version stored on edges.
    id: Option<&'static str>,
    side: Side,
    is_source: bool,
}

const fn handle(id: Option<&'static str>, side: Side, is_source: bool) -> HandleDef {
    HandleDef {
        id,
        side,
        is_source,
    }
}

fn handle_defs(node_type: NodeType) -> &'static [HandleDef] {
    const START: &[HandleDef] = &[handle(None, Side::Right, true)];
    const END: &[HandleDef] = &[handle(None, Side::Left, false)];
    const TEXT: &[HandleDef] = &[
        handle(Some("left-target"), Side::Left, false),
        handle(Some("top-target"), Side::Top, false),
        handle(Some("right-source"), Side::Right, true),
        handle(Some("bottom-source"), Side::Bottom, true),
    ];
    const CONTAINER: &[HandleDef] = &[handle(None, Side::Left, false)];
    const BRANCH: &[HandleDef] = &[handle(None, Side::Right, true)];
    match node_type {
        NodeType::Start => START,
        NodeType::End => END,
        NodeType::Text => TEXT,
        NodeType::BranchContainer => CONTAINER,
        NodeType::Branch | NodeType::SkillCheck => BRANCH,
    }
}

struct PlacedHandle {
    node: String,
    id: Option<String>,
    side: Side,
    pos: Pos2,
    is_source: bool,
}

struct Connecting {
    source: String,
    handle: Option<String>,
    side: Side,
    from: Pos2,
    to: Pos2,
}

enum NodeAction {
    Delete,
    AddBranch,
    AddSkillCheck,
}

#[derive(Default)]
pub struct GraphOutput {
    pub changed: bool,
}

/// Maps between world coordinates (where nodes live) and screen coordinates.
///
/// The canvas does its own pan and zoom instead of scaling a layer: node text
/// is laid out and rasterised at the zoomed size, so it stays sharp.
#[derive(Clone, Copy)]
struct Xf {
    canvas: Rect,
    /// The world position shown at the canvas's top-left corner.
    origin: Pos2,
    /// Screen points per world unit.
    zoom: f32,
}

impl Xf {
    fn to_screen(self, p: Pos2) -> Pos2 {
        self.canvas.min + (p - self.origin) * self.zoom
    }

    fn to_world(self, p: Pos2) -> Pos2 {
        self.origin + (p - self.canvas.min) / self.zoom
    }

    fn rect_to_screen(self, r: Rect) -> Rect {
        Rect::from_min_max(self.to_screen(r.min), self.to_screen(r.max))
    }

    /// The part of the world currently on screen.
    fn visible(self) -> Rect {
        Rect::from_min_size(self.origin, self.canvas.size() / self.zoom)
    }
}

/// Per-dialogue canvas state that isn't saved to the dialogue file.
pub struct GraphView {
    /// The world position shown at the canvas's top-left corner.
    origin: Pos2,
    zoom: f32,
    selected: Option<String>,
    connecting: Option<Connecting>,
    /// Node rectangles from the latest frame, in world coordinates.
    rects: HashMap<String, Rect>,
    context_pos: Option<Pos2>,
    spawn_count: u32,
    fitted: bool,
    /// The node being dragged on the canvas, while the pointer is down.
    dragging: Option<String>,
    /// How far a dragged branch has been pulled from its slot, in world units.
    /// It only changes order when released.
    reorder_offset: Vec2,
}

impl Default for GraphView {
    fn default() -> Self {
        Self {
            origin: Pos2::ZERO,
            zoom: 1.0,
            selected: None,
            connecting: None,
            rects: HashMap::new(),
            context_pos: None,
            spawn_count: 0,
            fitted: false,
            dragging: None,
            reorder_offset: Vec2::ZERO,
        }
    }
}

impl GraphView {
    /// Whether a node is selected, which is when the node panel shows.
    pub fn has_selection(&self, graph: &Graph) -> bool {
        self.selected
            .as_deref()
            .is_some_and(|id| graph.node(id).is_some())
    }

    fn xf(&self, canvas: Rect) -> Xf {
        Xf {
            canvas,
            origin: self.origin,
            zoom: self.zoom,
        }
    }

    /// Zooms by `factor`, keeping the world point under `at` (a screen
    /// position) where it is.
    fn zoom_about(&mut self, canvas: Rect, at: Pos2, factor: f32) {
        let anchor = self.xf(canvas).to_world(at);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.origin = anchor - (at - canvas.min) / self.zoom;
    }

    /// Frames `bounds`, with a margin, in the canvas.
    fn fit(&mut self, canvas: Rect, bounds: Rect) {
        let target = bounds.expand(60.0);
        let zoom = (canvas.size() / target.size()).min_elem();
        self.zoom = if zoom.is_finite() {
            zoom.clamp(MIN_ZOOM, MAX_FIT_ZOOM)
        } else {
            1.0
        };
        self.origin = target.center() - canvas.size() / self.zoom / 2.0;
    }

    /// Drag on the background (or middle-drag anywhere) pans; the scroll
    /// wheel and pinch zoom around the pointer.
    fn navigate(&mut self, ui: &Ui, canvas: Rect, response: &egui::Response) {
        if response.dragged_by(PointerButton::Primary) || response.dragged_by(PointerButton::Middle)
        {
            self.origin -= response.drag_delta() / self.zoom;
        }
        if response.contains_pointer()
            && let Some(at) = ui.input(|i| i.pointer.latest_pos())
        {
            let speed = ui.ctx().options(|o| o.input_options.scroll_zoom_speed);
            let factor = ui.input(|i| {
                i.zoom_delta() + (i.smooth_scroll_delta.x + i.smooth_scroll_delta.y) * speed
            });
            if factor != 1.0 && factor > 0.0 {
                self.zoom_about(canvas, at, factor);
            }
        }
    }

    /// `keyboard` is false while a modal is open, so Delete doesn't hit the
    /// canvas.
    pub fn show(&mut self, ui: &mut Ui, graph: &mut Graph, keyboard: bool) -> GraphOutput {
        let mut out = GraphOutput::default();
        let canvas = ui.available_rect_before_wrap();
        ui.painter().rect_filled(canvas, 0.0, theme::CANVAS_BG);
        // Registered before the nodes, so they sit on top of it and take
        // their own clicks and drags.
        let response = ui.allocate_rect(canvas, Sense::click_and_drag());
        self.navigate(ui, canvas, &response);

        let mut edge_paths: Vec<(String, [Pos2; 4])> = Vec::new();
        let mut canvas_ui = ui.new_child(UiBuilder::new().max_rect(canvas));
        canvas_ui.set_clip_rect(canvas.intersect(ui.clip_rect()));
        self.contents(&mut canvas_ui, canvas, graph, &mut out, &mut edge_paths);

        if !self.fitted {
            // Frame the whole dialogue (with a margin) the first time it's shown.
            self.fitted = true;
            if let Some(bounds) = self.bounds() {
                self.fit(canvas, bounds);
                ui.ctx().request_repaint();
            }
        }

        if response.clicked() {
            let hit = response
                .interact_pointer_pos()
                .and_then(|p| nearest_edge(&edge_paths, p, 6.0));
            match hit {
                Some(edge_id) => {
                    graph.disconnect(&edge_id);
                    out.changed = true;
                }
                None => self.selected = None,
            }
        }
        if response.secondary_clicked() {
            let xf = self.xf(canvas);
            self.context_pos = response.interact_pointer_pos().map(|p| xf.to_world(p));
        }
        response.context_menu(|ui| {
            for (name, node_type, _) in ADD_KINDS {
                if ui.button(format!("+ {name}")).clicked() {
                    let pos = self
                        .context_pos
                        .unwrap_or(self.xf(canvas).visible().center());
                    self.selected = Some(graph.add_node(*node_type, pos));
                    out.changed = true;
                    ui.close();
                }
            }
        });

        if keyboard
            && !ui.ctx().egui_wants_keyboard_input()
            && ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
            && let Some(id) = self.selected.take()
        {
            graph.remove_node(&id);
            out.changed = true;
        }

        // Single letters add a node, unless a text field wants the keys.
        if keyboard
            && !ui.ctx().egui_wants_keyboard_input()
            && let Some(node_type) = ui.input(|i| {
                if i.modifiers.any() {
                    return None;
                }
                ADD_KINDS
                    .iter()
                    .find(|(_, _, key)| i.key_pressed(*key))
                    .map(|(_, node_type, _)| *node_type)
            })
        {
            self.spawn(graph, canvas, node_type, &mut out);
        }

        self.toolbar(ui, canvas, graph, &mut out);
        self.minimap(ui, canvas);
        out
    }

    fn contents(
        &mut self,
        ui: &mut Ui,
        canvas: Rect,
        graph: &mut Graph,
        out: &mut GraphOutput,
        edge_paths: &mut Vec<(String, [Pos2; 4])>,
    ) {
        let xf = self.xf(canvas);
        let hover = ui.ctx().pointer_hover_pos();

        // A drag ends with the pointer. The frame it's released in still
        // applies the drop and draws the card where it was dropped, so the
        // state is only cleared after that.
        if !ui.input(|i| i.pointer.any_down() || i.pointer.any_released()) {
            self.dragging = None;
            self.reorder_offset = Vec2::ZERO;
        }

        paint_grid(ui.painter(), xf);
        let edge_layer = ui.painter().add(Shape::Noop);

        let mut handles = Vec::new();
        let mut release = None;
        let top_level: Vec<String> = graph
            .nodes
            .iter()
            .filter(|n| n.parent_id.as_ref().is_none_or(|p| graph.node(p).is_none()))
            .map(|n| n.id.clone())
            .collect();
        self.rects.retain(|id, _| graph.node(id).is_some());

        for id in &top_level {
            let mut ctx = NodeCtx {
                out: &mut *out,
                handles: &mut handles,
                release: &mut release,
            };
            let node = graph.node(id).expect("listed above");
            if node.node_type() == NodeType::BranchContainer {
                self.container_ui(ui, xf, graph, id, &mut ctx);
            } else {
                let origin = node.position;
                self.node_ui(
                    ui,
                    xf,
                    graph,
                    id,
                    origin,
                    node_width(node.node_type()),
                    id,
                    &mut ctx,
                );
            }
        }

        if let Some(at) = release
            && let Some(c) = self.connecting.take()
        {
            let target = handles
                .iter()
                .filter(|h| !h.is_source && h.node != c.source)
                .map(|h| (h, h.pos.distance(at)))
                .filter(|(_, d)| *d <= HANDLE_HIT * 2.0 / xf.zoom)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((t, _)) = target
                && graph.connect(&c.source, c.handle.clone(), &t.node, t.id.clone())
            {
                out.changed = true;
            }
        }
        if self.connecting.is_some() && !ui.input(|i| i.pointer.any_down()) {
            self.connecting = None;
        }

        if self.dragging.is_some() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
        }
        // While a node is dragged, its links stand out and the rest recede.
        // A container's links are its branches' links.
        let dragged = self.dragging.as_deref();
        let moves_with_drag = |node: &str| {
            dragged.is_some_and(|d| {
                node == d || graph.node(node).and_then(|n| n.parent_id.as_deref()) == Some(d)
            })
        };

        let arrow = ARROW_SIZE * xf.zoom;
        let mut shapes = Vec::new();
        for edge in &graph.edges {
            let find = |node: &str, handle: &Option<String>, source: bool| {
                handles
                    .iter()
                    .find(|h| h.node == node && h.is_source == source && &h.id == handle)
                    .or_else(|| {
                        handles
                            .iter()
                            .find(|h| h.node == node && h.is_source == source)
                    })
            };
            let (Some(src), Some(tgt)) = (
                find(&edge.source, &edge.source_handle, true),
                find(&edge.target, &edge.target_handle, false),
            ) else {
                continue;
            };
            // The curve is shaped in world units, then mapped to the screen.
            let points = bezier(src.pos, src.side, tgt.pos, tgt.side).map(|p| xf.to_screen(p));
            let hovered = self.connecting.is_none()
                && hover.is_some_and(|p| distance_to_curve(&points, p) <= 6.0);
            let attached = moves_with_drag(&edge.source) || moves_with_drag(&edge.target);
            let (color, width) = if attached || hovered {
                (theme::EDGE_HOVER, 2.5)
            } else if dragged.is_some() {
                (theme::EDGE.gamma_multiply(0.4), 1.5)
            } else {
                (theme::EDGE, 1.5)
            };
            // No arrowhead of its own: the input handle it ends at is one.
            push_edge(
                &mut shapes,
                points,
                tgt.side,
                color,
                (width * xf.zoom).max(1.0),
                0.0,
            );
            edge_paths.push((edge.id.clone(), points));
        }
        ui.painter().set(edge_layer, Shape::Vec(shapes));

        if let Some(c) = &self.connecting {
            let mut shapes = Vec::new();
            // The link snaps to the nearest handle it could land on.
            let snap = handles
                .iter()
                .filter(|h| !h.is_source && h.node != c.source)
                .map(|h| (h, h.pos.distance(c.to)))
                .filter(|(_, d)| *d <= HANDLE_HIT * 2.0 / xf.zoom)
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(h, _)| h);
            let (end, end_side) = match snap {
                Some(h) => (h.pos, h.side),
                None => (c.to, c.side.opposite()),
            };
            let points = bezier(c.from, c.side, end, end_side).map(|p| xf.to_screen(p));
            let color = graph
                .node(&c.source)
                .map_or(theme::EDGE_HOVER, |n| node_accent(n.node_type()));
            // Snapped onto an input, its own arrowhead does the pointing;
            // otherwise the loose end needs one.
            let arrow = if snap.is_some() { 0.0 } else { arrow };
            push_edge(
                &mut shapes,
                points,
                end_side,
                color,
                (1.5 * xf.zoom).max(1.0),
                arrow,
            );
            ui.painter().extend(shapes);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn node_ui(
        &mut self,
        ui: &mut Ui,
        xf: Xf,
        graph: &mut Graph,
        id: &str,
        origin: Pos2,
        width: f32,
        drag_target: &str,
        ctx: &mut NodeCtx<'_>,
    ) -> Rect {
        let egui_id = Id::unique(("dialnode-node", id));
        let (size, content) = {
            let node = graph.node(id).expect("caller checked");
            measure_card(ui, id, &node.data, width)
        };
        let rect = Rect::from_min_size(origin, size);
        self.rects.insert(id.to_string(), rect);
        // Where the card is in the layout; this is what picks up the drag.
        let slot = xf.rect_to_screen(rect);
        self.background_interaction(ui, xf, graph, id, slot, drag_target, ctx.out);

        let selected = self.selected.as_deref() == Some(id);
        let lifted = self.dragging.as_deref() == Some(id);
        // A branch being reordered is drawn where the pointer has taken it,
        // and its slot stays marked until it's dropped.
        let pulled = lifted && drag_target != id;
        let offset = if pulled {
            self.pulled_offset(graph, id)
        } else {
            Vec2::ZERO
        };
        let screen = xf.rect_to_screen(rect.translate(offset));
        if pulled {
            paint_empty_slot(ui.painter(), slot, xf.zoom, node_accent(NodeType::Branch));
        }

        let zoom = xf.zoom;
        let node = graph.node(id).expect("caller checked");
        let accent = node_accent(node.node_type());
        ui.scope_builder(
            UiBuilder::new()
                .id_salt(egui_id)
                .max_rect(Rect::from_min_size(
                    screen.min,
                    vec2(screen.width(), 10_000.0),
                )),
            |ui| {
                scale_style(ui, zoom);
                node_frame(selected, lifted, zoom, accent).show(ui, |ui| {
                    ui.set_width(content.x * zoom);
                    ui.set_min_height(content.y * zoom);
                    node_body(ui, &node.data);
                });
            },
        );
        // Over the dragged card, so the gap it will land in is always visible.
        if pulled && let Some((_, line_y)) = self.drop_target(graph, id) {
            paint_drop_line(ui.painter(), xf, rect, line_y);
        }
        self.handles_ui(ui, xf, graph, id, rect.translate(offset), ctx);
        rect
    }

    /// How far to draw a dragged branch from its slot: with the pointer, but
    /// kept inside its container and close to its column.
    fn pulled_offset(&self, graph: &Graph, child: &str) -> Vec2 {
        const SIDEWAYS: f32 = 20.0;
        let (Some(rect), Some(parent)) = (
            self.rects.get(child),
            graph
                .node(child)
                .and_then(|n| n.parent_id.as_deref())
                .and_then(|p| self.rects.get(p)),
        ) else {
            return Vec2::ZERO;
        };
        let top = parent.top() + CONTAINER_HEADER - 12.0 - rect.top();
        let bottom = parent.bottom() - CHILD_INSET + 12.0 - rect.bottom();
        let y = if bottom >= top {
            self.reorder_offset.y.clamp(top, bottom)
        } else {
            0.0
        };
        vec2(self.reorder_offset.x.clamp(-SIDEWAYS, SIDEWAYS), y)
    }

    /// Where a dragged branch would land if dropped now, as its index among
    /// its siblings, and the world y of the gap it would go in. `None` when
    /// it has no siblings to be ordered against.
    fn drop_target(&self, graph: &Graph, child: &str) -> Option<(usize, f32)> {
        let parent = graph.node(child)?.parent_id.clone()?;
        let rect = *self.rects.get(child)?;
        let centre = rect.center().y + self.pulled_offset(graph, child).y;
        let others: Vec<Rect> = graph
            .children_of(&parent)
            .iter()
            .filter(|sibling| sibling.as_str() != child)
            .filter_map(|sibling| self.rects.get(sibling).copied())
            .collect();
        if others.is_empty() {
            return None;
        }
        let index = others.iter().filter(|r| r.center().y < centre).count();
        let line_y = if index == 0 {
            others[0].top() - CHILD_GAP / 2.0
        } else if index == others.len() {
            others[index - 1].bottom() + CHILD_GAP / 2.0
        } else {
            (others[index - 1].bottom() + others[index].top()) / 2.0
        };
        Some((index, line_y))
    }

    fn container_ui(
        &mut self,
        ui: &mut Ui,
        xf: Xf,
        graph: &mut Graph,
        id: &str,
        ctx: &mut NodeCtx<'_>,
    ) {
        let node = graph.node(id).expect("caller checked");
        let origin = node.position;
        let width = node.width.unwrap_or(0.0).max(BRANCH_CONTAINER_MIN_WIDTH);
        let height = node.height.unwrap_or(0.0).max(BRANCH_CONTAINER_MIN_HEIGHT);
        let rect = Rect::from_min_size(origin, vec2(width, height));
        let screen = xf.rect_to_screen(rect);
        let zoom = xf.zoom;
        let selected = self.selected.as_deref() == Some(id);
        let lifted = self.dragging.as_deref() == Some(id);
        let accent = node_accent(NodeType::BranchContainer);

        self.background_interaction(ui, xf, graph, id, screen, id, ctx.out);
        ui.scope_builder(
            UiBuilder::new()
                .id_salt(Id::unique(("dialnode-node", id)))
                .max_rect(screen),
            |ui| {
                scale_style(ui, zoom);
                node_frame(selected, lifted, zoom, accent).show(ui, |ui| {
                    ui.set_min_size((rect.size() - Vec2::splat(2.0 * NODE_MARGIN)) * zoom);
                    ui.set_max_width((rect.width() - 2.0 * NODE_MARGIN) * zoom);
                    // Just its name: branches are added from the inspector.
                    ui.style_mut().interaction.selectable_labels = false;
                    ui.label(
                        RichText::new(node_title(NodeType::BranchContainer))
                            .strong()
                            .color(accent),
                    );
                });
            },
        );
        self.rects.insert(id.to_string(), rect);

        let mut y = origin.y + CONTAINER_HEADER;
        // A branch being dragged is drawn last, so it passes over the others.
        let mut pulled = None;
        for child in graph.children_of(id) {
            let child_origin = pos2(origin.x + CHILD_INSET, y);
            let child_width = width - 2.0 * CHILD_INSET;
            let child_type = graph.node(&child).expect("listed").node_type();
            if !child_type.is_child() {
                continue;
            }
            if self.dragging.as_deref() == Some(child.as_str()) {
                // Its slot keeps its place in the layout until it's dropped.
                let size = {
                    let node = graph.node(&child).expect("listed");
                    measure_card(ui, &child, &node.data, child_width).0
                };
                y = Rect::from_min_size(child_origin, size).bottom() + CHILD_GAP;
                self.rects
                    .insert(child.clone(), Rect::from_min_size(child_origin, size));
                pulled = Some((child, child_origin, child_width));
                continue;
            }
            let child_rect =
                self.node_ui(ui, xf, graph, &child, child_origin, child_width, id, ctx);
            y = child_rect.bottom() + CHILD_GAP;
        }
        if let Some((child, child_origin, child_width)) = pulled {
            self.node_ui(ui, xf, graph, &child, child_origin, child_width, id, ctx);
        }

        let needed = y - origin.y - CHILD_GAP + CHILD_INSET;
        if needed > height + 0.5 {
            graph.node_mut(id).expect("exists").height = Some(needed);
            ui.ctx().request_repaint();
        }

        if selected {
            let grip = Rect::from_min_size(screen.max - Vec2::splat(14.0), Vec2::splat(14.0));
            let resp = ui
                .interact(grip, Id::unique(("dialnode-resize", id)), Sense::drag())
                .on_hover_cursor(egui::CursorIcon::ResizeNwSe);
            let painter = ui.painter();
            for i in 1..=3 {
                let o = i as f32 * 4.0;
                painter.line_segment(
                    [
                        pos2(grip.max.x - o, grip.max.y - 2.0),
                        pos2(grip.max.x - 2.0, grip.max.y - o),
                    ],
                    Stroke::new(1.0, theme::SMALL_TEXT),
                );
            }
            if resp.dragged() {
                let delta = resp.drag_delta() / zoom;
                let node = graph.node_mut(id).expect("exists");
                node.width = Some((width + delta.x).max(BRANCH_CONTAINER_MIN_WIDTH));
                node.height = Some(
                    (height + delta.y)
                        .max(needed)
                        .max(BRANCH_CONTAINER_MIN_HEIGHT),
                );
                ctx.out.changed = true;
            }
        }

        self.handles_ui(ui, xf, graph, id, rect, ctx);
    }

    /// Click-to-select and drag-to-move, registered under the node's widgets
    /// so the card's buttons still get their clicks. `rect` is in screen
    /// coordinates.
    ///
    /// Dragging a branch inside a container pulls it out of its slot, and the
    /// order only changes when it's dropped; the container itself is dragged
    /// by its own background.
    #[allow(clippy::too_many_arguments)]
    fn background_interaction(
        &mut self,
        ui: &mut Ui,
        xf: Xf,
        graph: &mut Graph,
        id: &str,
        rect: Rect,
        drag_target: &str,
        out: &mut GraphOutput,
    ) {
        let resp = ui.interact(
            rect,
            Id::unique(("dialnode-node-bg", id)),
            Sense::click_and_drag(),
        );
        if resp.clicked() || resp.drag_started() {
            self.selected = Some(id.to_string());
        }
        // An open hand over a card that can be picked up.
        if self.dragging.is_none() && resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
        }
        if resp.dragged() {
            self.dragging = Some(id.to_string());
        }

        if drag_target != id {
            if resp.drag_started() {
                self.reorder_offset = Vec2::ZERO;
            }
            if resp.dragged() {
                self.reorder_offset += resp.drag_delta() / xf.zoom;
            }
            if resp.drag_stopped()
                && let Some((index, _)) = self.drop_target(graph, id)
                && graph.move_child(id, index)
            {
                out.changed = true;
                ui.ctx().request_repaint();
            }
            return;
        }
        let delta = resp.drag_delta() / xf.zoom;
        if delta != Vec2::ZERO
            && let Some(node) = graph.node_mut(drag_target)
        {
            node.position += delta;
            out.changed = true;
        }
    }

    /// `rect` is the node's rectangle in world coordinates.
    fn handles_ui(
        &mut self,
        ui: &mut Ui,
        xf: Xf,
        graph: &Graph,
        id: &str,
        rect: Rect,
        ctx: &mut NodeCtx<'_>,
    ) {
        let node_type = graph.node(id).expect("caller checked").node_type();
        let defs = handle_defs(node_type);
        // A text node only offers the source handle it's already using.
        let used_source = graph
            .edges
            .iter()
            .find(|e| e.source == id)
            .and_then(|e| e.source_handle.clone())
            .filter(|h| defs.iter().any(|d| d.is_source && d.id == Some(h.as_str())));

        for def in defs {
            if def.is_source
                && let Some(used) = &used_source
                && def.id != Some(used.as_str())
            {
                continue;
            }
            // The handle sits just outside the card, on a short stub, so it
            // never covers the card's text.
            let anchor = def.side.anchor(rect);
            let pos = anchor + def.side.outward() * HANDLE_OFFSET;
            ctx.handles.push(PlacedHandle {
                node: id.to_string(),
                id: def.id.map(String::from),
                side: def.side,
                pos,
                is_source: def.is_source,
            });

            let sense = if def.is_source {
                Sense::drag()
            } else {
                Sense::hover()
            };
            let screen_pos = xf.to_screen(pos);
            let hit = (2.0 * HANDLE_HIT * xf.zoom).clamp(14.0, 40.0);
            let resp = ui.interact(
                Rect::from_center_size(screen_pos, Vec2::splat(hit)),
                Id::unique(("dialnode-handle", id, def.side as u8)),
                sense,
            );
            // Linked as the edges are drawn: an edge uses the handle it names,
            // or the first of that kind when it names none that exist.
            let first_of_kind = defs.iter().find(|d| d.is_source == def.is_source);
            let is_first = first_of_kind.is_some_and(|d| std::ptr::eq(d, def));
            let linked = graph.edges.iter().any(|e| {
                let (node, named) = if def.is_source {
                    (&e.source, e.source_handle.as_deref())
                } else {
                    (&e.target, e.target_handle.as_deref())
                };
                node == id
                    && (named == def.id
                        || (is_first
                            && !defs
                                .iter()
                                .any(|d| d.is_source == def.is_source && d.id == named)))
            });
            // While a link is being dragged, the handles that can take it are
            // the ones that matter; otherwise a node shows its open handles
            // when you're near it or have it selected.
            let linking = self.connecting.as_ref();
            let can_take_link = !def.is_source && linking.is_some_and(|c| c.source != id);
            let starts_link = linking.is_some_and(|c| c.source == id && c.side == def.side);
            let near_card = ui.ctx().pointer_hover_pos().is_some_and(|p| {
                xf.rect_to_screen(rect)
                    .expand((HANDLE_OFFSET + HANDLE_HIT) * xf.zoom)
                    .contains(p)
            });
            let selected = self.selected.as_deref() == Some(id);
            let visible = linked
                || can_take_link
                || starts_link
                || (linking.is_none() && (selected || near_card));
            if visible {
                // Handles take their node's colour. Direction is told by
                // shape: an output is a circle, an input an arrowhead.
                let color = node_accent(node_type);
                let snapping = can_take_link
                    && linking.is_some_and(|c| c.to.distance(pos) <= HANDLE_HIT * 2.0 / xf.zoom);
                let radius = (HANDLE_RADIUS * xf.zoom).clamp(3.0, 9.0);
                let radius = radius
                    + if snapping {
                        3.0
                    } else if resp.hovered() {
                        1.5
                    } else {
                        0.0
                    };
                let painter = ui.painter();
                painter.line_segment(
                    [xf.to_screen(anchor), screen_pos],
                    Stroke::new((1.5 * xf.zoom).max(1.0), theme::EDGE),
                );
                if can_take_link {
                    // A glow marks the handles this link can land on.
                    let glow = if snapping { 0.45 } else { 0.2 };
                    painter.circle_filled(screen_pos, radius + 5.0, color.gamma_multiply(glow));
                }
                // Open handles are hollow; linked ones are solid.
                let solid = linked || snapping || starts_link;
                paint_handle(
                    painter,
                    screen_pos,
                    def.side,
                    def.is_source,
                    radius,
                    color,
                    solid,
                );
            }

            if !def.is_source {
                continue;
            }
            if resp.drag_started() {
                self.connecting = Some(Connecting {
                    source: id.to_string(),
                    handle: def.id.map(String::from),
                    side: def.side,
                    from: pos,
                    to: pos,
                });
            }
            if let Some(c) = &mut self.connecting
                && c.source == id
                && c.side == def.side
            {
                c.from = pos;
                if let Some(p) = resp.interact_pointer_pos() {
                    c.to = xf.to_world(p);
                }
                if resp.drag_stopped() {
                    *ctx.release = Some(c.to);
                }
            }
        }
    }

    fn apply(&mut self, graph: &mut Graph, id: &str, action: NodeAction, out: &mut GraphOutput) {
        match action {
            NodeAction::Delete => {
                graph.remove_node(id);
                if self.selected.as_deref() == Some(id) {
                    self.selected = None;
                }
                out.changed = true;
            }
            NodeAction::AddBranch => {
                graph.add_child(id, NodeType::Branch);
                out.changed = true;
            }
            NodeAction::AddSkillCheck => {
                graph.add_child(id, NodeType::SkillCheck);
                out.changed = true;
            }
        }
    }

    /// Adds a node. With a node selected it goes just after it, and is linked
    /// to it when that's possible; otherwise it goes near the middle of the
    /// view.
    fn spawn(
        &mut self,
        graph: &mut Graph,
        canvas: Rect,
        node_type: NodeType,
        out: &mut GraphOutput,
    ) {
        let placement = self.place_after_selected(graph, node_type);
        let (pos, link) = match placement {
            Some(Placement { pos, link }) => (pos, link),
            None => {
                let offset = (self.spawn_count % 8) as f32 * 24.0;
                self.spawn_count += 1;
                let centre = self.xf(canvas).visible().center();
                let size = vec2(node_width(node_type), nominal_height(node_type));
                let pos = centre + vec2(offset - 80.0, offset - 40.0);
                (self.clear_of_nodes(pos, size), None)
            }
        };
        let id = graph.add_node(node_type, pos);
        if let Some(link) = link {
            graph.connect(&link.source, link.source_handle, &id, link.target_handle);
        }
        let size = vec2(node_width(node_type), nominal_height(node_type));
        self.reveal(canvas, Rect::from_min_size(pos, size));
        self.selected = Some(id);
        out.changed = true;
    }

    /// Moves `pos` down until a node of `size` there wouldn't sit on another.
    fn clear_of_nodes(&self, mut pos: Pos2, size: Vec2) -> Pos2 {
        for _ in 0..32 {
            let wanted = Rect::from_min_size(pos, size).expand(8.0);
            if self.rects.values().any(|r| r.intersects(wanted)) {
                pos.y += size.y + 24.0;
            } else {
                break;
            }
        }
        pos
    }

    /// Where to put a new node so it follows the selected one: to the right,
    /// level with it, and clear of other nodes. The link is only offered when
    /// the selected node can have one and doesn't already, since connecting
    /// would replace its existing link.
    fn place_after_selected(&self, graph: &Graph, new_type: NodeType) -> Option<Placement> {
        // A start node begins a dialogue, so it isn't added after anything.
        if new_type == NodeType::Start {
            return None;
        }
        let selected = self.selected.as_deref()?;
        let node = graph.node(selected)?;
        // Branches and skill checks sit in a container, so go after that.
        let anchor = *self
            .rects
            .get(node.parent_id.as_deref().unwrap_or(selected))?;
        let row = self.rects.get(selected).map_or(anchor.top(), |r| r.top());

        let size = vec2(node_width(new_type), nominal_height(new_type));
        let pos = self.clear_of_nodes(pos2(anchor.right() + SPAWN_GAP, row), size);

        let source_handle = match node.node_type() {
            NodeType::Text => Some("right-source".to_string()),
            NodeType::Start | NodeType::Branch | NodeType::SkillCheck => None,
            NodeType::End | NodeType::BranchContainer => {
                return Some(Placement { pos, link: None });
            }
        };
        let has_link = graph.edges.iter().any(|e| e.source == selected);
        let link = (!has_link).then(|| Link {
            source: selected.to_string(),
            source_handle,
            target_handle: (new_type == NodeType::Text).then(|| "left-target".to_string()),
        });
        Some(Placement { pos, link })
    }

    /// Pans just enough to bring `rect` (in world coordinates) into view.
    fn reveal(&mut self, canvas: Rect, rect: Rect) {
        const MARGIN: f32 = 40.0;
        let visible = self.xf(canvas).visible();
        let margin = MARGIN / self.zoom;
        let shift = |lo: f32, hi: f32, view_lo: f32, view_hi: f32| {
            if lo < view_lo + margin {
                lo - view_lo - margin
            } else if hi > view_hi - margin {
                hi - view_hi + margin
            } else {
                0.0
            }
        };
        self.origin += vec2(
            shift(rect.left(), rect.right(), visible.left(), visible.right()),
            shift(rect.top(), rect.bottom(), visible.top(), visible.bottom()),
        );
    }

    /// Adds a node with its top-left corner at `pos`, unlinked, and selects it.
    fn spawn_at(
        &mut self,
        graph: &mut Graph,
        node_type: NodeType,
        pos: Pos2,
        out: &mut GraphOutput,
    ) {
        self.selected = Some(graph.add_node(node_type, pos));
        out.changed = true;
    }

    /// Click a button to add a node after the selected one, or drag it onto
    /// the canvas to drop it exactly where you want it.
    fn toolbar(&mut self, ui: &Ui, canvas: Rect, graph: &mut Graph, out: &mut GraphOutput) {
        let xf = self.xf(canvas);
        let canvas_layer = ui.layer_id();
        let mut clicked = None;
        let mut dropped = None;
        Area::new(Id::unique("graph-toolbar"))
            .order(Order::Middle)
            .fixed_pos(pos2(canvas.center().x, canvas.top() + 10.0))
            .pivot(Align2::CENTER_TOP)
            .show(ui.ctx(), |ui| {
                overlay_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (name, node_type, key) in ADD_KINDS {
                            let accent = node_accent(*node_type);
                            let response = theme::icon_tile(ui, |painter, centre| {
                                paint_node_icon(painter, centre, *node_type, accent);
                            });
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Button,
                                    true,
                                    format!("Add {name}"),
                                )
                            });
                            let response = response.on_hover_text(format!(
                                "Add {name} ({}), or drag it onto the canvas",
                                key.name()
                            ));
                            if response.clicked() {
                                clicked = Some(*node_type);
                            }
                            if response.dragged() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                if let Some(at) = ui.ctx().pointer_latest_pos() {
                                    paint_drag_ghost(ui.ctx(), xf, *node_type, at);
                                }
                            }
                            if response.drag_stopped() {
                                dropped = ui.ctx().pointer_latest_pos().map(|at| (*node_type, at));
                            }
                        }
                    });
                });
            });

        if let Some(node_type) = clicked {
            self.spawn(graph, canvas, node_type, out);
        }
        // Only a drop on the bare canvas counts: not on the toolbar, the
        // minimap or anything else floating above it.
        if let Some((node_type, at)) = dropped
            && canvas.contains(at)
            && ui.ctx().layer_id_at(at) == Some(canvas_layer)
        {
            let size = vec2(node_width(node_type), nominal_height(node_type));
            self.spawn_at(graph, node_type, xf.to_world(at) - size / 2.0, out);
        }
    }

    fn minimap(&mut self, ui: &Ui, canvas: Rect) {
        let Some(bounds) = self.bounds() else {
            return;
        };
        let visible = self.xf(canvas).visible();
        let world = bounds.union(visible);
        Area::new(Id::unique("graph-minimap"))
            .order(Order::Middle)
            .fixed_pos(canvas.right_bottom() - vec2(10.0, 10.0))
            .pivot(Align2::RIGHT_BOTTOM)
            .show(ui.ctx(), |ui| {
                overlay_frame().inner_margin(4).show(ui, |ui| {
                    let size = vec2(200.0, 140.0);
                    let (rect, resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
                    let scale = (size.x / world.width()).min(size.y / world.height());
                    let offset = rect.center() - world.center().to_vec2() * scale;
                    let to_map = |p: Pos2| (p.to_vec2() * scale + offset.to_vec2()).to_pos2();
                    let painter = ui.painter_at(rect);
                    for r in self.rects.values() {
                        painter.rect_filled(
                            Rect::from_min_max(to_map(r.min), to_map(r.max)),
                            2.0,
                            theme::MINIMAP_NODE,
                        );
                    }
                    painter.rect_stroke(
                        Rect::from_min_max(to_map(visible.min), to_map(visible.max)),
                        0.0,
                        Stroke::new(1.0, theme::PRIMARY),
                        egui::StrokeKind::Inside,
                    );
                    if (resp.clicked() || resp.dragged())
                        && let Some(p) = resp.interact_pointer_pos()
                    {
                        let world_pos = ((p - offset) / scale).to_pos2();
                        self.origin = world_pos - visible.size() / 2.0;
                    }
                });
            });
    }

    fn bounds(&self) -> Option<Rect> {
        self.rects.values().copied().reduce(|a, b| a.union(b))
    }

    /// The panel that edits the selected node; draws nothing without a
    /// selection. Returns whether the graph changed.
    pub fn inspector(&mut self, ui: &mut Ui, graph: &mut Graph) -> bool {
        let Some(node) = self.selected.as_deref().and_then(|id| graph.node_mut(id)) else {
            return false;
        };
        let id = node.id.clone();
        let mut changed = false;
        let mut action = None;

        ui.add_space(8.0);
        ui.heading(
            RichText::new(node_title(node.node_type())).color(node_accent(node.node_type())),
        );
        // Field state (cursor, scroll) belongs to one node.
        ui.push_id(&id, |ui| match &mut node.data {
            NodeData::Start(_) | NodeData::End => {
                ui.add_space(6.0);
                ui.label(RichText::new("This node has nothing to edit.").color(theme::SMALL_TEXT));
            }
            NodeData::Text(d) => {
                changed |= optional_field(ui, "Speaker", &mut d.speaker);
                changed |= text_field(ui, "Text", &mut d.text, 8);
            }
            NodeData::BranchContainer(_) => {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if theme::secondary_button(ui, "+ Branch").clicked() {
                        action = Some(NodeAction::AddBranch);
                    }
                    if theme::secondary_button(ui, "+ Skill check").clicked() {
                        action = Some(NodeAction::AddSkillCheck);
                    }
                });
            }
            NodeData::Branch(d) => changed |= text_field(ui, "Text", &mut d.text, 3),
            NodeData::SkillCheck(d) => {
                changed |= text_field(ui, "Text", &mut d.text, 3);
                changed |= optional_field(ui, "Skill", &mut d.skill);
                let label = field_label(ui, "Difficulty");
                changed |= ui
                    .add(DragValue::new(&mut d.difficulty).range(1..=20))
                    .labelled_by(label.id)
                    .changed();
            }
        });

        ui.add_space(16.0);
        if theme::secondary_button(ui, "Delete Node").clicked() {
            action = Some(NodeAction::Delete);
        }
        let mut out = GraphOutput { changed };
        if let Some(action) = action {
            self.apply(graph, &id, action, &mut out);
        }
        out.changed
    }
}

/// The nodes you can add from the toolbar, the context menu and the keyboard.
const ADD_KINDS: &[(&str, NodeType, Key)] = &[
    ("Start", NodeType::Start, Key::S),
    ("Node", NodeType::Text, Key::N),
    ("Branch", NodeType::BranchContainer, Key::B),
    ("End", NodeType::End, Key::E),
];

/// The colour that marks a kind of node, on its toolbar button and its cards.
fn node_accent(node_type: NodeType) -> Color32 {
    match node_type {
        NodeType::Start => theme::NODE_START,
        NodeType::Text => theme::NODE_TEXT,
        NodeType::BranchContainer | NodeType::Branch => theme::NODE_BRANCH,
        NodeType::SkillCheck => theme::NODE_SKILL,
        NodeType::End => theme::NODE_END,
    }
}

/// Paints a toolbar icon for a kind of node, centred on `c`.
fn paint_node_icon(painter: &egui::Painter, c: Pos2, node_type: NodeType, color: Color32) {
    let stroke = Stroke::new(1.6, color);
    let thin = Stroke::new(1.2, color);
    match node_type {
        NodeType::Start => {
            painter.add(Shape::convex_polygon(
                vec![
                    c + vec2(-4.5, -7.0),
                    c + vec2(-4.5, 7.0),
                    c + vec2(7.0, 0.0),
                ],
                color,
                Stroke::NONE,
            ));
        }
        NodeType::Text | NodeType::SkillCheck => {
            // A speech bubble with two lines of text.
            painter.rect_stroke(
                Rect::from_min_max(c + vec2(-9.0, -7.0), c + vec2(9.0, 4.0)),
                3.0,
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.add(Shape::line(
                vec![c + vec2(-4.0, 4.0), c + vec2(-4.0, 8.0), c + vec2(0.0, 4.0)],
                stroke,
            ));
            painter.line_segment([c + vec2(-5.0, -3.0), c + vec2(5.0, -3.0)], thin);
            painter.line_segment([c + vec2(-5.0, 0.5), c + vec2(2.0, 0.5)], thin);
        }
        NodeType::BranchContainer | NodeType::Branch => {
            // One line splitting into two.
            let root = c + vec2(-8.0, 0.0);
            let split = c + vec2(-1.0, 0.0);
            let (top, bottom) = (c + vec2(8.0, -6.0), c + vec2(8.0, 6.0));
            painter.line_segment([root, split], stroke);
            for end in [top, bottom] {
                painter.add(Shape::line(
                    vec![split, pos2(c.x + 2.0, end.y), end],
                    stroke,
                ));
                painter.circle_filled(end, 2.2, color);
            }
            painter.circle_filled(root, 2.2, color);
        }
        NodeType::End => {
            painter.rect_filled(Rect::from_center_size(c, Vec2::splat(11.0)), 2.5, color);
        }
    }
}

/// An outline of the node under the cursor while one is dragged from the
/// toolbar, showing where and how big it will land.
fn paint_drag_ghost(ctx: &egui::Context, xf: Xf, node_type: NodeType, at: Pos2) {
    let painter = ctx.layer_painter(LayerId::new(
        Order::Tooltip,
        Id::unique("dialnode-node-ghost"),
    ));
    let accent = node_accent(node_type);
    let size = vec2(node_width(node_type), nominal_height(node_type)) * xf.zoom;
    let rect = Rect::from_center_size(at, size);
    let radius = scaled_radius(CornerRadius::same(12), xf.zoom);
    painter.rect(
        rect,
        radius,
        accent.gamma_multiply(0.14),
        Stroke::new(1.5, accent),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        node_title(node_type),
        FontId::proportional((15.0 * xf.zoom).clamp(8.0, 40.0)),
        accent,
    );
}

/// Space left between a node and the one added after it.
const SPAWN_GAP: f32 = 80.0;

/// A rough height for a node that hasn't been laid out yet, to keep new nodes
/// from landing on top of old ones.
fn nominal_height(node_type: NodeType) -> f32 {
    match node_type {
        NodeType::Start | NodeType::End => 44.0,
        NodeType::BranchContainer => BRANCH_CONTAINER_MIN_HEIGHT,
        NodeType::Text => 90.0,
        _ => 60.0,
    }
}

/// Where a new node goes when it's added after the selected one, and the link
/// that joins them.
struct Placement {
    pos: Pos2,
    link: Option<Link>,
}

struct Link {
    source: String,
    source_handle: Option<String>,
    target_handle: Option<String>,
}

struct NodeCtx<'a> {
    out: &'a mut GraphOutput,
    handles: &'a mut Vec<PlacedHandle>,
    release: &'a mut Option<Pos2>,
}

fn node_width(node_type: NodeType) -> f32 {
    match node_type {
        NodeType::Start | NodeType::End => 130.0,
        NodeType::Text => 280.0,
        _ => 220.0,
    }
}

/// A node card's frame, drawn at `zoom`. A `lifted` card, one being dragged,
/// looks picked up: brighter, with a firm outline and a deep shadow.
fn node_frame(selected: bool, lifted: bool, zoom: f32, accent: Color32) -> Frame {
    // A resting card has a border faintly tinted with its colour.
    let stroke = if lifted {
        Stroke::new(2.0 * zoom, accent)
    } else if selected {
        Stroke::new(1.5 * zoom, accent)
    } else {
        Stroke::new(zoom, theme::NODE_BORDER.lerp_to_gamma(accent, 0.3))
    };
    let (fill, shadow) = if lifted {
        (
            theme::NODE_BG.lerp_to_gamma(Color32::WHITE, 0.07),
            Shadow {
                offset: [0, (12.0 * zoom).round() as i8],
                blur: (28.0 * zoom).round() as u8,
                spread: (2.0 * zoom).round() as u8,
                color: Color32::from_black_alpha(170),
            },
        )
    } else {
        (
            theme::NODE_BG,
            Shadow {
                offset: [0, (4.0 * zoom).round() as i8],
                blur: (8.0 * zoom).round() as u8,
                spread: 0,
                color: Color32::from_black_alpha(110),
            },
        )
    };
    Frame::new()
        .fill(fill)
        .stroke(stroke)
        .corner_radius(scaled_radius(CornerRadius::same(12), zoom))
        .inner_margin(NODE_MARGIN * zoom)
        .shadow(shadow)
}

/// A dashed outline marking the slot a dragged branch came from.
fn paint_empty_slot(painter: &egui::Painter, slot: Rect, zoom: f32, color: Color32) {
    let stroke = Stroke::new(1.2, color.gamma_multiply(0.7));
    let corner = 12.0 * zoom;
    // Straight dashes along each side, kept off the rounded corners.
    let inset = corner.min(slot.width() / 3.0).min(slot.height() / 3.0);
    let (l, r, t, b) = (slot.left(), slot.right(), slot.top(), slot.bottom());
    let sides = [
        [pos2(l + inset, t), pos2(r - inset, t)],
        [pos2(l + inset, b), pos2(r - inset, b)],
        [pos2(l, t + inset), pos2(l, b - inset)],
        [pos2(r, t + inset), pos2(r, b - inset)],
    ];
    for side in sides {
        painter.extend(Shape::dashed_line(&side, stroke, 6.0, 4.0));
    }
    painter.rect_filled(slot, corner, color.gamma_multiply(0.05));
}

/// The line showing where a dragged branch will land, across the width of its
/// column at world height `y`.
fn paint_drop_line(painter: &egui::Painter, xf: Xf, column: Rect, y: f32) {
    let color = theme::PRIMARY;
    let from = xf.to_screen(pos2(column.left(), y));
    let to = xf.to_screen(pos2(column.right(), y));
    painter.line_segment([from, to], Stroke::new(2.5, color));
    for end in [from, to] {
        painter.circle_filled(end, 3.5, color);
    }
}

fn scaled_radius(radius: CornerRadius, zoom: f32) -> CornerRadius {
    let scale = |r: u8| (f32::from(r) * zoom).round().clamp(0.0, 255.0) as u8;
    CornerRadius {
        nw: scale(radius.nw),
        ne: scale(radius.ne),
        sw: scale(radius.sw),
        se: scale(radius.se),
    }
}

/// Scales a `Ui`'s fonts, spacing and widget shapes by `zoom`. A card drawn
/// in such a `Ui` is laid out and rasterised at the zoomed size, rather than
/// being a 1:1 drawing stretched afterwards, which is what keeps its text
/// sharp.
fn scale_style(ui: &mut Ui, zoom: f32) {
    let style = ui.style_mut();
    for font in style.text_styles.values_mut() {
        // Whole and half sizes only, so zooming doesn't fill the glyph atlas
        // with near-duplicates.
        font.size = ((font.size * zoom * 2.0).round() / 2.0).max(1.0);
    }
    let spacing = &mut style.spacing;
    spacing.item_spacing *= zoom;
    spacing.button_padding *= zoom;
    spacing.interact_size *= zoom;
    spacing.icon_width *= zoom;
    spacing.icon_width_inner *= zoom;
    spacing.icon_spacing *= zoom;
    spacing.indent *= zoom;
    let widgets = &mut style.visuals.widgets;
    for state in [
        &mut widgets.noninteractive,
        &mut widgets.inactive,
        &mut widgets.hovered,
        &mut widgets.active,
        &mut widgets.open,
    ] {
        state.corner_radius = scaled_radius(state.corner_radius, zoom);
        state.bg_stroke.width *= zoom;
        state.fg_stroke.width *= zoom;
        state.expansion *= zoom;
    }
}

/// A card's size at 1:1, as `(outer, content)`. Layout is kept in world
/// units, so it's measured here in an invisible pass at 1:1 whatever the zoom
/// is; the visible card is then drawn at that size scaled up or down. This
/// keeps a node's size, and the container heights derived from it, from
/// drifting as font rounding changes with the zoom.
fn measure_card(ui: &mut Ui, id: &str, data: &NodeData, width: f32) -> (Vec2, Vec2) {
    let mut probe = ui.new_child(
        UiBuilder::new()
            .id_salt(("dialnode-measure", id))
            .max_rect(Rect::from_min_size(
                pos2(-1.0e6, -1.0e6),
                vec2(width, 10_000.0),
            ))
            .invisible(),
    );
    let mut content = Vec2::ZERO;
    let outer = node_frame(false, false, 1.0, node_accent(data.node_type()))
        .show(&mut probe, |ui| {
            ui.set_width(width - 2.0 * NODE_MARGIN);
            node_body(ui, data);
            content = ui.min_rect().size();
        })
        .response
        .rect
        .size();
    (outer, content)
}

fn overlay_frame() -> Frame {
    Frame::new()
        .fill(theme::OVERLAY_BG)
        .stroke(Stroke::new(1.0, theme::NODE_BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(6)
}

fn node_title(node_type: NodeType) -> &'static str {
    match node_type {
        NodeType::Start => "Start",
        NodeType::End => "End",
        NodeType::Text => "Node",
        NodeType::BranchContainer => "Branches",
        NodeType::Branch => "Branch",
        NodeType::SkillCheck => "Skill Check",
    }
}

/// A read-only summary of a node for its card. Editing, and deleting, happen
/// in the inspector, so there's nothing on the card to click by accident.
fn node_body(ui: &mut Ui, data: &NodeData) {
    // Plain labels so clicks and drags reach the card underneath.
    ui.style_mut().interaction.selectable_labels = false;
    let title = RichText::new(node_title(data.node_type()))
        .strong()
        .color(node_accent(data.node_type()));
    // Start and End cards have nothing but their name, so it sits in the middle.
    if matches!(data, NodeData::Start(_) | NodeData::End) {
        ui.vertical_centered(|ui| ui.label(title));
    } else {
        ui.label(title);
    }

    let preview = |ui: &mut Ui, text: &str| {
        if text.trim().is_empty() {
            ui.label(
                RichText::new("No text yet")
                    .italics()
                    .color(theme::SMALL_TEXT),
            );
        } else {
            ui.add(Label::new(text).wrap());
        }
    };
    match data {
        NodeData::Start(_) | NodeData::End | NodeData::BranchContainer(_) => {}
        NodeData::Text(d) => {
            if let Some(speaker) = &d.speaker {
                ui.label(RichText::new(speaker).strong().color(theme::PRIMARY));
            }
            preview(ui, &d.text);
        }
        NodeData::Branch(d) => preview(ui, &d.text),
        NodeData::SkillCheck(d) => {
            preview(ui, &d.text);
            let skill = d.skill.as_deref().unwrap_or("No skill");
            ui.label(
                RichText::new(format!("{skill} · Difficulty {}", d.difficulty))
                    .small()
                    .color(theme::SMALL_TEXT),
            );
        }
    }
}

/// A field's caption in the inspector.
fn field_label(ui: &mut Ui, text: &str) -> egui::Response {
    ui.add_space(6.0);
    ui.label(RichText::new(text).small().color(theme::SMALL_TEXT))
}

/// A text field in the inspector.
fn text_field(ui: &mut Ui, label: &str, value: &mut String, rows: usize) -> bool {
    let label = field_label(ui, label);
    let edit = if rows > 1 {
        TextEdit::multiline(value).desired_rows(rows)
    } else {
        TextEdit::singleline(value)
    };
    ui.scope(|ui| {
        theme::outline_fields(ui);
        ui.add(edit.desired_width(f32::INFINITY))
            .labelled_by(label.id)
            .changed()
    })
    .inner
}

/// A one-line text field for an optional value; empty means `None`.
fn optional_field(ui: &mut Ui, label: &str, value: &mut Option<String>) -> bool {
    let mut text = value.clone().unwrap_or_default();
    let changed = text_field(ui, label, &mut text, 1);
    if changed {
        *value = (!text.is_empty()).then_some(text);
    }
    changed
}

fn paint_grid(painter: &egui::Painter, xf: Xf) {
    // Double the spacing until the lines are comfortably apart on screen.
    let mut step = GRID_SPACING;
    while step * xf.zoom < 12.0 {
        step *= 2.0;
    }
    let clip = painter.clip_rect();
    let world = Rect::from_min_max(xf.to_world(clip.min), xf.to_world(clip.max));
    if (world.width() + world.height()) / step > 2_000.0 {
        return;
    }

    // Lines sit on whole pixels so they stay crisp, and every fifth one is a
    // little stronger to give the grid some structure.
    let line = |from: Pos2, to: Pos2, index: i64| {
        let color = if index % GRID_MAJOR_EVERY == 0 {
            theme::CANVAS_GRID_MAJOR
        } else {
            theme::CANVAS_GRID
        };
        Shape::line_segment([from, to], Stroke::new(1.0, color))
    };
    let first = |from: f32| (from / step).floor() as i64;
    let mut shapes = Vec::new();
    for i in first(world.left())..=(world.right() / step).ceil() as i64 {
        let x = painter.round_to_pixel_center(xf.to_screen(pos2(i as f32 * step, 0.0)).x);
        shapes.push(line(pos2(x, clip.top()), pos2(x, clip.bottom()), i));
    }
    for j in first(world.top())..=(world.bottom() / step).ceil() as i64 {
        let y = painter.round_to_pixel_center(xf.to_screen(pos2(0.0, j as f32 * step)).y);
        shapes.push(line(pos2(clip.left(), y), pos2(clip.right(), y), j));
    }
    painter.extend(shapes);
}

/// Control points matching xyflow's `getBezierPath`.
fn bezier(from: Pos2, from_side: Side, to: Pos2, to_side: Side) -> [Pos2; 4] {
    fn offset(distance: f32) -> f32 {
        if distance >= 0.0 {
            0.5 * distance
        } else {
            CURVATURE * 25.0 * (-distance).sqrt()
        }
    }
    fn control(pos: Pos2, side: Side, other: Pos2) -> Pos2 {
        match side {
            Side::Left => pos2(pos.x - offset(pos.x - other.x), pos.y),
            Side::Right => pos2(pos.x + offset(other.x - pos.x), pos.y),
            Side::Top => pos2(pos.x, pos.y - offset(pos.y - other.y)),
            Side::Bottom => pos2(pos.x, pos.y + offset(other.y - pos.y)),
        }
    }
    [
        from,
        control(from, from_side, to),
        control(to, to_side, from),
        to,
    ]
}

fn bezier_point(p: &[Pos2; 4], t: f32) -> Pos2 {
    let u = 1.0 - t;
    let v = p[0].to_vec2() * (u * u * u)
        + p[1].to_vec2() * (3.0 * u * u * t)
        + p[2].to_vec2() * (3.0 * u * t * t)
        + p[3].to_vec2() * (t * t * t);
    v.to_pos2()
}

fn distance_to_curve(points: &[Pos2; 4], p: Pos2) -> f32 {
    const STEPS: usize = 24;
    let mut best = f32::INFINITY;
    let mut prev = points[0];
    for i in 1..=STEPS {
        let next = bezier_point(points, i as f32 / STEPS as f32);
        best = best.min(distance_to_segment(p, prev, next));
        prev = next;
    }
    best
}

fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let t = if ab.length_sq() > 0.0 {
        ((p - a).dot(ab) / ab.length_sq()).clamp(0.0, 1.0)
    } else {
        0.0
    };
    p.distance(a + ab * t)
}

fn nearest_edge(paths: &[(String, [Pos2; 4])], p: Pos2, tolerance: f32) -> Option<String> {
    paths
        .iter()
        .map(|(id, pts)| (id, distance_to_curve(pts, p)))
        .filter(|(_, d)| *d <= tolerance)
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(id, _)| id.clone())
}

/// Draws a connection handle on `side` of a card. An output (`is_source`) is a
/// circle; an input is an arrowhead pointing into the card. Solid when linked
/// (or about to be), hollow when open.
fn paint_handle(
    painter: &egui::Painter,
    at: Pos2,
    side: Side,
    is_source: bool,
    radius: f32,
    color: Color32,
    solid: bool,
) {
    let (fill, stroke) = if solid {
        (color, Stroke::new(1.5, theme::NODE_BG))
    } else {
        (theme::NODE_BG, Stroke::new(1.5, color))
    };
    if is_source {
        painter.circle(at, radius, fill, stroke);
        return;
    }
    let inward = -side.outward();
    let across = vec2(-inward.y, inward.x);
    let tip = at + inward * (radius * 1.3);
    let back = at - inward * (radius * 0.9);
    painter.add(Shape::convex_polygon(
        vec![
            tip,
            back + across * (radius * 1.1),
            back - across * (radius * 1.1),
        ],
        fill,
        stroke,
    ));
}

/// Pushes a curve plus an arrowhead pointing into the target handle. The
/// points, `width` and `arrow` size are in screen units. With an `arrow` of
/// zero the curve just ends, for when the target's own handle is the arrow.
fn push_edge(
    shapes: &mut Vec<Shape>,
    mut points: [Pos2; 4],
    target_side: Side,
    color: Color32,
    width: f32,
    arrow: f32,
) {
    let tip = points[3];
    let back = target_side.outward();
    let base = tip + back * arrow;
    points[3] = base;
    points[2] += back * arrow;
    shapes.push(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke::new(width, color),
    )));
    if arrow > 0.0 {
        let perp = vec2(-back.y, back.x) * (arrow * 0.5);
        shapes.push(Shape::convex_polygon(
            vec![tip, base + perp, base - perp],
            color,
            Stroke::NONE,
        ));
    }
}

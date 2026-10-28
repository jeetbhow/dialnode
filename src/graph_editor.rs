//! The node-graph canvas: pan/zoom, node widgets, handles, and edges.

use std::collections::HashMap;

use eframe::egui::{
    self, Align, Align2, Area, Color32, ComboBox, CornerRadius, DragValue, Frame, Id, Key, Layout,
    Order, Pos2, Rect, Scene, Sense, Shadow, Shape, Stroke, TextEdit, TextStyle, Ui, UiBuilder,
    Vec2, containers::DragPanButtons, epaint::CubicBezierShape, pos2, vec2,
};

use crate::images::TextureCache;
use crate::model::{
    BRANCH_CONTAINER_MIN_HEIGHT, BRANCH_CONTAINER_MIN_WIDTH, Graph, NodeData, NodeType, Skill,
};
use crate::theme;

const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_HIT: f32 = 10.0;
const CURVATURE: f32 = 0.4;
const ARROW_SIZE: f32 = 10.0;
const CONTAINER_HEADER: f32 = 44.0;
const CHILD_INSET: f32 = 12.0;
const CHILD_GAP: f32 = 8.0;
const NODE_MARGIN: f32 = 8.0;
const GRID_SPACING: f32 = 20.0;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityRequestKind {
    Speaker,
    Portrait,
}

pub enum GraphEvent {
    RequestEntity {
        node_id: String,
        kind: EntityRequestKind,
    },
    Error(String),
}

enum NodeAction {
    Delete,
    AddBranch,
    AddSkillCheck,
    Request(EntityRequestKind),
}

pub struct GraphEnv<'a> {
    pub skills: &'a [Skill],
    pub textures: &'a mut TextureCache,
    /// False while a modal is open, so Delete doesn't hit the canvas.
    pub keyboard: bool,
}

#[derive(Default)]
pub struct GraphOutput {
    pub changed: bool,
    pub events: Vec<GraphEvent>,
}

/// Per-dialogue canvas state that isn't saved to the database.
pub struct GraphView {
    scene_rect: Rect,
    selected: Option<String>,
    connecting: Option<Connecting>,
    /// Node rectangles from the latest frame, in scene coordinates.
    rects: HashMap<String, Rect>,
    /// Whether the speaker / portrait fields are shown on a text node.
    text_fields: HashMap<String, (bool, bool)>,
    context_pos: Option<Pos2>,
    spawn_count: u32,
    fitted: bool,
}

impl Default for GraphView {
    fn default() -> Self {
        Self {
            // A zero rect makes the scene fit its contents on first show.
            scene_rect: Rect::ZERO,
            selected: None,
            connecting: None,
            rects: HashMap::new(),
            text_fields: HashMap::new(),
            context_pos: None,
            spawn_count: 0,
            fitted: false,
        }
    }
}

impl GraphView {
    pub fn show_text_field(&mut self, node_id: &str, kind: EntityRequestKind) {
        let fields = self.text_fields.entry(node_id.to_string()).or_default();
        match kind {
            EntityRequestKind::Speaker => fields.0 = true,
            EntityRequestKind::Portrait => fields.1 = true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, graph: &mut Graph, mut env: GraphEnv<'_>) -> GraphOutput {
        let mut out = GraphOutput::default();
        let canvas = ui.available_rect_before_wrap();
        ui.painter().rect_filled(canvas, 0.0, theme::CANVAS_BG);

        let mut scene_rect = self.scene_rect;
        let mut edge_paths: Vec<(String, [Pos2; 4])> = Vec::new();
        let mut actions: Vec<(String, NodeAction)> = Vec::new();
        let response = Scene::new()
            .zoom_range(0.1..=2.0)
            .scroll_zooms(true)
            .drag_pan_buttons(DragPanButtons::PRIMARY | DragPanButtons::MIDDLE)
            .show(ui, &mut scene_rect, |ui| {
                self.contents(ui, graph, &mut env, &mut out, &mut edge_paths, &mut actions);
            })
            .response;
        self.scene_rect = scene_rect;
        if !self.fitted {
            // Frame the whole dialogue (with a margin) the first time it's shown.
            self.fitted = true;
            if let Some(bounds) = self.bounds() {
                self.scene_rect = bounds.expand(60.0);
                ui.ctx().request_repaint();
            }
        }

        for (id, action) in actions {
            self.apply(graph, &id, action, env.skills, &mut out);
        }

        let zoom = canvas.width() / self.scene_rect.width().max(1.0);
        if response.clicked() {
            let hit = response
                .interact_pointer_pos()
                .and_then(|p| nearest_edge(&edge_paths, p, 6.0 / zoom));
            match hit {
                Some(edge_id) => {
                    graph.disconnect(&edge_id);
                    out.changed = true;
                }
                None => self.selected = None,
            }
        }
        if response.secondary_clicked() {
            self.context_pos = response.interact_pointer_pos();
        }
        response.context_menu(|ui| {
            for (label, node_type) in ADD_BUTTONS {
                if ui.button(*label).clicked() {
                    let pos = self.context_pos.unwrap_or(self.scene_rect.center());
                    self.selected = Some(graph.add_node(*node_type, pos));
                    out.changed = true;
                    ui.close();
                }
            }
        });

        if env.keyboard
            && !ui.ctx().egui_wants_keyboard_input()
            && ui.input(|i| i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace))
            && let Some(id) = self.selected.take()
        {
            graph.remove_node(&id);
            out.changed = true;
        }

        self.toolbar(ui, canvas, graph, &mut out);
        self.controls(ui, canvas);
        self.minimap(ui, canvas);
        out
    }

    fn contents(
        &mut self,
        ui: &mut Ui,
        graph: &mut Graph,
        env: &mut GraphEnv<'_>,
        out: &mut GraphOutput,
        edge_paths: &mut Vec<(String, [Pos2; 4])>,
        actions: &mut Vec<(String, NodeAction)>,
    ) {
        let from_global = ui
            .ctx()
            .layer_transform_from_global(ui.layer_id())
            .unwrap_or_default();
        let hover = ui.ctx().pointer_hover_pos().map(|p| from_global * p);
        let zoom = 1.0 / from_global.scaling;

        paint_grid(ui.painter(), ui.clip_rect());
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
                env: &mut *env,
                out: &mut *out,
                handles: &mut handles,
                release: &mut release,
                actions: &mut *actions,
            };
            let node = graph.node(id).expect("listed above");
            if node.node_type() == NodeType::BranchContainer {
                self.container_ui(ui, graph, id, &mut ctx);
            } else {
                let origin = node.position;
                self.node_ui(
                    ui,
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
                .filter(|(_, d)| *d <= HANDLE_HIT * 2.0)
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
            let points = bezier(src.pos, src.side, tgt.pos, tgt.side);
            let hovered = self.connecting.is_none()
                && hover.is_some_and(|p| distance_to_curve(&points, p) <= 6.0 / zoom);
            let color = if hovered {
                theme::EDGE_HOVER
            } else {
                theme::EDGE
            };
            push_edge(
                &mut shapes,
                points,
                tgt.side,
                color,
                if hovered { 2.5 } else { 1.5 },
            );
            edge_paths.push((edge.id.clone(), points));
        }
        ui.painter().set(edge_layer, Shape::Vec(shapes));

        if let Some(c) = &self.connecting {
            let mut shapes = Vec::new();
            let points = bezier(c.from, c.side, c.to, c.side.opposite());
            push_edge(
                &mut shapes,
                points,
                c.side.opposite(),
                theme::SOURCE_HANDLE,
                1.5,
            );
            ui.painter().extend(shapes);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn node_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut Graph,
        id: &str,
        origin: Pos2,
        width: f32,
        drag_target: &str,
        ctx: &mut NodeCtx<'_, '_>,
    ) -> Rect {
        let egui_id = Id::unique(("dialnode-node", id));
        let size = self
            .rects
            .get(id)
            .map(|r| r.size())
            .unwrap_or(vec2(width, 60.0));
        self.background_interaction(
            ui,
            graph,
            id,
            Rect::from_min_size(origin, size),
            drag_target,
            ctx.out,
        );

        let selected = self.selected.as_deref() == Some(id);
        let fields = self.text_fields.entry(id.to_string()).or_insert_with(|| {
            match graph.node(id).map(|n| &n.data) {
                Some(NodeData::Text(d)) => (d.speaker.is_some(), d.portrait.is_some()),
                _ => (false, false),
            }
        });
        let node = graph.node_mut(id).expect("caller checked");
        let rect = ui
            .scope_builder(
                UiBuilder::new()
                    .id_salt(egui_id)
                    .max_rect(Rect::from_min_size(origin, vec2(width, 10_000.0))),
                |ui| {
                    node_frame(selected).show(ui, |ui| {
                        ui.set_width(width - 2.0 * NODE_MARGIN);
                        let action = node_body(
                            ui,
                            id,
                            &mut node.data,
                            fields,
                            ctx.env,
                            &mut ctx.out.changed,
                        );
                        if let Some(action) = action {
                            ctx.actions.push((id.to_string(), action));
                        }
                    });
                },
            )
            .response
            .rect;
        self.rects.insert(id.to_string(), rect);
        self.handles_ui(ui, graph, id, rect, ctx);
        rect
    }

    fn container_ui(
        &mut self,
        ui: &mut Ui,
        graph: &mut Graph,
        id: &str,
        ctx: &mut NodeCtx<'_, '_>,
    ) {
        let node = graph.node(id).expect("caller checked");
        let origin = node.position;
        let width = node.width.unwrap_or(0.0).max(BRANCH_CONTAINER_MIN_WIDTH);
        let height = node.height.unwrap_or(0.0).max(BRANCH_CONTAINER_MIN_HEIGHT);
        let rect = Rect::from_min_size(origin, vec2(width, height));
        let selected = self.selected.as_deref() == Some(id);

        self.background_interaction(ui, graph, id, rect, id, ctx.out);
        ui.scope_builder(
            UiBuilder::new()
                .id_salt(Id::unique(("dialnode-node", id)))
                .max_rect(rect),
            |ui| {
                node_frame(selected).show(ui, |ui| {
                    ui.set_min_size(rect.size() - Vec2::splat(2.0 * NODE_MARGIN));
                    ui.set_max_width(rect.width() - 2.0 * NODE_MARGIN);
                    ui.horizontal(|ui| {
                        if ui
                            .button("+ Branch")
                            .on_hover_text("Add a branch")
                            .clicked()
                        {
                            ctx.actions.push((id.to_string(), NodeAction::AddBranch));
                        }
                        if ui
                            .button("+ Skill check")
                            .on_hover_text("Add a skill check")
                            .clicked()
                        {
                            ctx.actions
                                .push((id.to_string(), NodeAction::AddSkillCheck));
                        }
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .add(theme::icon_button("×"))
                                .on_hover_text("Delete")
                                .clicked()
                            {
                                ctx.actions.push((id.to_string(), NodeAction::Delete));
                            }
                        });
                    });
                });
            },
        );
        self.rects.insert(id.to_string(), rect);

        let mut y = origin.y + CONTAINER_HEADER;
        for child in graph.children_of(id) {
            let child_origin = pos2(origin.x + CHILD_INSET, y);
            let child_width = width - 2.0 * CHILD_INSET;
            let child_type = graph.node(&child).expect("listed").node_type();
            if !child_type.is_child() {
                continue;
            }
            let child_rect = self.node_ui(ui, graph, &child, child_origin, child_width, id, ctx);
            if let Some(n) = graph.node_mut(&child) {
                // Keep the stored position relative to the parent, as before.
                n.position = (child_origin - origin).to_pos2();
            }
            y = child_rect.bottom() + CHILD_GAP;
        }

        let needed = y - origin.y - CHILD_GAP + CHILD_INSET;
        if needed > height + 0.5 {
            graph.node_mut(id).expect("exists").height = Some(needed);
            ui.ctx().request_repaint();
        }

        if selected {
            let grip = Rect::from_min_size(rect.max - Vec2::splat(14.0), Vec2::splat(14.0));
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
                let delta = resp.drag_delta();
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

        self.handles_ui(ui, graph, id, rect, ctx);
    }

    /// Click-to-select and drag-to-move, registered under the node's widgets
    /// so text fields and buttons still get their clicks.
    fn background_interaction(
        &mut self,
        ui: &mut Ui,
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
        let delta = resp.drag_delta();
        if delta != Vec2::ZERO
            && let Some(node) = graph.node_mut(drag_target)
        {
            node.position += delta;
            out.changed = true;
        }
    }

    fn handles_ui(
        &mut self,
        ui: &mut Ui,
        graph: &Graph,
        id: &str,
        rect: Rect,
        ctx: &mut NodeCtx<'_, '_>,
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
            let pos = def.side.anchor(rect);
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
            let resp = ui.interact(
                Rect::from_center_size(pos, Vec2::splat(HANDLE_HIT * 2.0)),
                Id::unique(("dialnode-handle", id, def.side as u8)),
                sense,
            );
            let color = if def.is_source {
                theme::SOURCE_HANDLE
            } else {
                theme::TARGET_HANDLE
            };
            let radius = if resp.hovered() {
                HANDLE_RADIUS + 1.5
            } else {
                HANDLE_RADIUS
            };
            ui.painter()
                .circle(pos, radius, color, Stroke::new(1.0, Color32::WHITE));

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
                    c.to = p;
                }
                if resp.drag_stopped() {
                    *ctx.release = Some(c.to);
                }
            }
        }
    }

    fn apply(
        &mut self,
        graph: &mut Graph,
        id: &str,
        action: NodeAction,
        skills: &[Skill],
        out: &mut GraphOutput,
    ) {
        match action {
            NodeAction::Delete => {
                graph.remove_node(id);
                if self.selected.as_deref() == Some(id) {
                    self.selected = None;
                }
                out.changed = true;
            }
            NodeAction::AddBranch => {
                graph.add_child(id, NodeType::Branch, None);
                out.changed = true;
            }
            NodeAction::AddSkillCheck => {
                let Some(skill) = skills.first().cloned() else {
                    out.events.push(GraphEvent::Error(
                        "Skill checks require at least 1 skill in the database.".into(),
                    ));
                    return;
                };
                graph.add_child(id, NodeType::SkillCheck, Some(skill));
                out.changed = true;
            }
            NodeAction::Request(kind) => out.events.push(GraphEvent::RequestEntity {
                node_id: id.to_string(),
                kind,
            }),
        }
    }

    fn toolbar(&mut self, ui: &Ui, canvas: Rect, graph: &mut Graph, out: &mut GraphOutput) {
        Area::new(Id::unique("graph-toolbar"))
            .order(Order::Middle)
            .fixed_pos(pos2(canvas.center().x, canvas.top() + 10.0))
            .pivot(Align2::CENTER_TOP)
            .show(ui.ctx(), |ui| {
                overlay_frame().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (label, node_type) in ADD_BUTTONS {
                            if ui.add(theme::secondary_button(label)).clicked() {
                                let offset = (self.spawn_count % 8) as f32 * 24.0;
                                self.spawn_count += 1;
                                let pos =
                                    self.scene_rect.center() + vec2(offset - 80.0, offset - 40.0);
                                self.selected = Some(graph.add_node(*node_type, pos));
                                out.changed = true;
                            }
                        }
                    });
                });
            });
    }

    fn controls(&mut self, ui: &Ui, canvas: Rect) {
        Area::new(Id::unique("graph-controls"))
            .order(Order::Middle)
            .fixed_pos(pos2(canvas.left() + 10.0, canvas.bottom() - 10.0))
            .pivot(Align2::LEFT_BOTTOM)
            .show(ui.ctx(), |ui| {
                overlay_frame().show(ui, |ui| {
                    ui.vertical(|ui| {
                        let center = self.scene_rect.center();
                        if ui.button("+").on_hover_text("Zoom in").clicked() {
                            self.scene_rect =
                                Rect::from_center_size(center, self.scene_rect.size() * 0.8);
                        }
                        if ui.button("-").on_hover_text("Zoom out").clicked() {
                            self.scene_rect =
                                Rect::from_center_size(center, self.scene_rect.size() * 1.25);
                        }
                        if ui.button("Fit").on_hover_text("Fit view").clicked()
                            && let Some(bounds) = self.bounds()
                        {
                            self.scene_rect = bounds.expand(60.0);
                        }
                    });
                });
            });
    }

    fn minimap(&mut self, ui: &Ui, canvas: Rect) {
        let Some(bounds) = self.bounds() else {
            return;
        };
        let world = bounds.union(self.scene_rect);
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
                            Color32::from_gray(200),
                        );
                    }
                    painter.rect_stroke(
                        Rect::from_min_max(
                            to_map(self.scene_rect.min),
                            to_map(self.scene_rect.max),
                        ),
                        0.0,
                        Stroke::new(1.0, theme::PRIMARY),
                        egui::StrokeKind::Inside,
                    );
                    if (resp.clicked() || resp.dragged())
                        && let Some(p) = resp.interact_pointer_pos()
                    {
                        let world_pos = ((p - offset) / scale).to_pos2();
                        self.scene_rect = Rect::from_center_size(world_pos, self.scene_rect.size());
                    }
                });
            });
    }

    fn bounds(&self) -> Option<Rect> {
        self.rects.values().copied().reduce(|a, b| a.union(b))
    }
}

const ADD_BUTTONS: &[(&str, NodeType)] = &[
    ("+ Start", NodeType::Start),
    ("+ Node", NodeType::Text),
    ("+ Branch", NodeType::BranchContainer),
    ("+ End", NodeType::End),
];

struct NodeCtx<'a, 'e> {
    env: &'a mut GraphEnv<'e>,
    out: &'a mut GraphOutput,
    handles: &'a mut Vec<PlacedHandle>,
    release: &'a mut Option<Pos2>,
    actions: &'a mut Vec<(String, NodeAction)>,
}

fn node_width(node_type: NodeType) -> f32 {
    match node_type {
        NodeType::Start | NodeType::End => 130.0,
        NodeType::Text => 280.0,
        _ => 220.0,
    }
}

fn node_frame(selected: bool) -> Frame {
    let stroke = if selected {
        Stroke::new(1.5, theme::PRIMARY)
    } else {
        Stroke::new(1.0, theme::NODE_BORDER)
    };
    Frame::new()
        .fill(theme::NODE_BG)
        .stroke(stroke)
        .corner_radius(CornerRadius::same(12))
        .inner_margin(NODE_MARGIN)
        .shadow(Shadow {
            offset: [0, 4],
            blur: 8,
            spread: 0,
            color: Color32::from_black_alpha(40),
        })
}

fn overlay_frame() -> Frame {
    Frame::new()
        .fill(Color32::from_white_alpha(235))
        .stroke(Stroke::new(1.0, theme::NODE_BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(6)
}

/// The widgets inside a node. Only touches the node's own data; anything that
/// changes the graph structure is returned as an action.
fn node_body(
    ui: &mut Ui,
    id: &str,
    data: &mut NodeData,
    fields: &mut (bool, bool),
    env: &mut GraphEnv<'_>,
    changed: &mut bool,
) -> Option<NodeAction> {
    let mut action = None;
    let header = |ui: &mut Ui, title: &str, action: &mut Option<NodeAction>| {
        ui.horizontal(|ui| {
            ui.strong(title);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(theme::icon_button("×"))
                    .on_hover_text("Delete")
                    .clicked()
                {
                    *action = Some(NodeAction::Delete);
                }
            });
        });
    };

    match data {
        NodeData::Start(_) => header(ui, "Start", &mut action),
        NodeData::End => header(ui, "End", &mut action),
        NodeData::Text(d) => {
            ui.horizontal(|ui| {
                ui.menu_button("…", |ui| {
                    ui.checkbox(&mut fields.0, "Speaker");
                    ui.checkbox(&mut fields.1, "Portrait");
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(theme::icon_button("×"))
                        .on_hover_text("Delete")
                        .clicked()
                    {
                        action = Some(NodeAction::Delete);
                    }
                });
            });

            if fields.1 {
                ui.horizontal(|ui| match &d.portrait {
                    Some(portrait) => {
                        match env.textures.get(ui.ctx(), &portrait.id, &portrait.data_url) {
                            Some(texture) => {
                                ui.add(
                                    egui::Image::new(&texture).fit_to_exact_size(Vec2::splat(50.0)),
                                )
                                .on_hover_text(&portrait.name);
                            }
                            None => {
                                ui.label(&portrait.name);
                            }
                        }
                        if ui
                            .small_button("-")
                            .on_hover_text("Remove portrait")
                            .clicked()
                        {
                            d.portrait = None;
                            *changed = true;
                        }
                    }
                    None => {
                        ui.label("Portrait:");
                        if ui
                            .small_button("+")
                            .on_hover_text("Choose a portrait")
                            .clicked()
                        {
                            action = Some(NodeAction::Request(EntityRequestKind::Portrait));
                        }
                    }
                });
            }
            if fields.0 {
                ui.horizontal(|ui| match &d.speaker {
                    Some(speaker) => {
                        ui.label(&speaker.name);
                        if ui
                            .small_button("-")
                            .on_hover_text("Remove speaker")
                            .clicked()
                        {
                            d.speaker = None;
                            *changed = true;
                        }
                    }
                    None => {
                        ui.label("Speaker:");
                        if ui
                            .small_button("+")
                            .on_hover_text("Choose a speaker")
                            .clicked()
                        {
                            action = Some(NodeAction::Request(EntityRequestKind::Speaker));
                        }
                    }
                });
            }
            let resp = ui.add(
                TextEdit::multiline(&mut d.text)
                    .hint_text("Enter the dialogue text.")
                    .font(TextStyle::Monospace)
                    .desired_rows(2)
                    .desired_width(f32::INFINITY),
            );
            *changed |= resp.changed();
        }
        NodeData::BranchContainer(_) => {}
        NodeData::Branch(d) => {
            header(ui, "Branch", &mut action);
            ui.label("Text:");
            let resp = ui.add(
                TextEdit::singleline(&mut d.text)
                    .hint_text("Branch text")
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
            *changed |= resp.changed();
        }
        NodeData::SkillCheck(d) => {
            header(ui, "Skill Check", &mut action);
            ui.label("Text:");
            let resp = ui.add(
                TextEdit::singleline(&mut d.text)
                    .hint_text("Enter skill check text.")
                    .font(TextStyle::Monospace)
                    .desired_width(f32::INFINITY),
            );
            *changed |= resp.changed();

            ui.horizontal(|ui| {
                let label = ui.label("Skill:");
                let current = d
                    .skill
                    .as_ref()
                    .map_or("(none)", |s| s.name.as_str())
                    .to_string();
                ComboBox::from_id_salt(("skill-check-skill", id))
                    .selected_text(current)
                    .show_ui(ui, |ui| {
                        for skill in env.skills {
                            let is_current = d.skill.as_ref().is_some_and(|s| s.id == skill.id);
                            if ui.selectable_label(is_current, &skill.name).clicked() && !is_current
                            {
                                d.skill = Some(skill.clone());
                                *changed = true;
                            }
                        }
                    })
                    .response
                    .labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                let label = ui.label("Difficulty:");
                let resp = ui
                    .add(DragValue::new(&mut d.difficulty).range(1..=20))
                    .labelled_by(label.id);
                *changed |= resp.changed();
            });
        }
    }
    action
}

fn paint_grid(painter: &egui::Painter, clip: Rect) {
    let cols = (clip.width() / GRID_SPACING) as usize;
    let rows = (clip.height() / GRID_SPACING) as usize;
    if cols * rows > 20_000 {
        return;
    }
    let start_x = (clip.left() / GRID_SPACING).floor() * GRID_SPACING;
    let start_y = (clip.top() / GRID_SPACING).floor() * GRID_SPACING;
    let mut shapes = Vec::with_capacity((cols + 2) * (rows + 2));
    let mut y = start_y;
    while y <= clip.bottom() {
        let mut x = start_x;
        while x <= clip.right() {
            shapes.push(Shape::circle_filled(pos2(x, y), 1.0, theme::CANVAS_DOT));
            x += GRID_SPACING;
        }
        y += GRID_SPACING;
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

/// Pushes a curve plus an arrowhead pointing into the target handle.
fn push_edge(
    shapes: &mut Vec<Shape>,
    mut points: [Pos2; 4],
    target_side: Side,
    color: Color32,
    width: f32,
) {
    let tip = points[3];
    let back = target_side.outward();
    let base = tip + back * ARROW_SIZE;
    points[3] = base;
    points[2] += back * ARROW_SIZE;
    shapes.push(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
        points,
        false,
        Color32::TRANSPARENT,
        Stroke::new(width, color),
    )));
    let perp = vec2(-back.y, back.x) * (ARROW_SIZE * 0.5);
    shapes.push(Shape::convex_polygon(
        vec![tip, base + perp, base - perp],
        color,
        Stroke::NONE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    struct State {
        graph: Graph,
        view: GraphView,
        textures: TextureCache,
    }

    const SIZE: Vec2 = vec2(1000.0, 700.0);

    fn harness(graph: Graph) -> Harness<'static, State> {
        let state = State {
            graph,
            view: GraphView::default(),
            textures: TextureCache::default(),
        };
        let mut harness = Harness::builder().with_size(SIZE).build_ui_state(
            |ui, s: &mut State| {
                egui::CentralPanel::no_frame().show(ui, |ui| {
                    s.view.show(
                        ui,
                        &mut s.graph,
                        GraphEnv {
                            skills: &[],
                            textures: &mut s.textures,
                            keyboard: true,
                        },
                    );
                });
            },
            state,
        );
        harness.run();
        harness
    }

    /// Mirrors how `Scene` fits `scene_rect` into the canvas.
    fn to_screen(view: &GraphView, p: Pos2) -> Pos2 {
        let canvas = Rect::from_min_size(Pos2::ZERO, SIZE);
        let scale = (canvas.size() / view.scene_rect.size()).min_elem();
        (canvas.center().to_vec2() + (p.to_vec2() - view.scene_rect.center().to_vec2()) * scale)
            .to_pos2()
    }

    fn drag(harness: &mut Harness<'_, State>, from: Pos2, to: Pos2) {
        harness.hover_at(from);
        harness.step();
        harness.drag_at(from);
        harness.step();
        for i in 1..=5 {
            harness.hover_at(from + (to - from) * (i as f32 / 5.0));
            harness.step();
        }
        harness.drop_at(to);
        harness.run();
    }

    #[test]
    fn dragging_between_handles_connects_nodes() {
        let mut graph = Graph::default();
        let start = graph.add_node(NodeType::Start, pos2(0.0, 0.0));
        let text = graph.add_node(NodeType::Text, pos2(300.0, 0.0));
        let mut h = harness(graph);

        let view = &h.state().view;
        let from = to_screen(view, view.rects[&start].right_center());
        let to = to_screen(view, view.rects[&text].left_center());
        drag(&mut h, from, to);

        let g = &h.state().graph;
        assert_eq!(g.edges.len(), 1, "expected an edge");
        assert_eq!(g.edges[0].target_handle.as_deref(), Some("left-target"));
        assert_eq!(
            g.node(&start).unwrap().data,
            NodeData::Start(crate::model::StartData { next: Some(text) })
        );
    }

    #[test]
    fn dragging_a_node_moves_it_and_delete_removes_it() {
        let mut graph = Graph::default();
        let end = graph.add_node(NodeType::End, pos2(0.0, 0.0));
        graph.add_node(NodeType::Start, pos2(300.0, 200.0));
        let mut h = harness(graph);

        let view = &h.state().view;
        // Grab the node near its left edge, away from the label and the delete button.
        let grab = to_screen(view, view.rects[&end].left_center() + vec2(6.0, 0.0));
        drag(&mut h, grab, grab + vec2(50.0, 30.0));
        let moved = h.state().graph.node(&end).unwrap().position;
        assert!(
            moved.x > 10.0 && moved.y > 5.0,
            "node did not move: {moved:?}"
        );

        h.key_press(Key::Delete);
        h.run();
        assert!(h.state().graph.node(&end).is_none());
    }

    #[test]
    fn edge_hit_testing() {
        let pts = bezier(pos2(0.0, 0.0), Side::Right, pos2(100.0, 0.0), Side::Left);
        assert!(distance_to_curve(&pts, pos2(50.0, 2.0)) < 3.0);
        assert!(distance_to_curve(&pts, pos2(50.0, 40.0)) > 30.0);
        let paths = vec![("e".to_string(), pts)];
        assert_eq!(
            nearest_edge(&paths, pos2(50.0, 1.0), 6.0).as_deref(),
            Some("e")
        );
        assert_eq!(nearest_edge(&paths, pos2(50.0, 20.0), 6.0), None);
    }
}

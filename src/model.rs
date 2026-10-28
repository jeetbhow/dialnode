//! Core data types. Node data mirrors the dialogue file format: speakers,
//! portraits, and skills are referenced by name / `res://` path.

use eframe::egui::{Pos2, pos2};
use serde::{Deserialize, Serialize};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FORMAT_VERSION: &str = "2.0.0";

pub const ROOT_ID: &str = "root";
pub const BRANCH_CONTAINER_MIN_WIDTH: f32 = 240.0;
pub const BRANCH_CONTAINER_MIN_HEIGHT: f32 = 240.0;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// The contents of `manifest.dial`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub godot_project_location: Option<String>,
    pub created_on: String,
    pub app_version: String,
    pub format_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Speaker {
    pub id: String,
    pub name: String,
}

impl Speaker {
    pub fn new(name: String) -> Self {
        Self { id: new_id(), name }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Portrait {
    pub id: String,
    pub name: String,
    /// Path of the image relative to the Godot project.
    pub rel_path: String,
    /// `res://` path, which is what dialogue files reference.
    pub virtual_path: String,
    pub filename: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Skill {
    pub id: String,
    pub name: String,
}

impl Skill {
    pub fn new(name: String) -> Self {
        Self { id: new_id(), name }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SkillCategory {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub skills: Vec<Skill>,
}

/// The contents of `database.json`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Entities {
    #[serde(default)]
    pub speakers: Vec<Speaker>,
    #[serde(default)]
    pub skill_categories: Vec<SkillCategory>,
    #[serde(default)]
    pub portraits: Vec<Portrait>,
}

impl Entities {
    pub fn all_skills(&self) -> Vec<Skill> {
        self.skill_categories
            .iter()
            .flat_map(|c| c.skills.iter().cloned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeType {
    Start,
    End,
    Text,
    BranchContainer,
    Branch,
    SkillCheck,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeType::Start => "start",
            NodeType::End => "end",
            NodeType::Text => "text",
            NodeType::BranchContainer => "branchContainer",
            NodeType::Branch => "branch",
            NodeType::SkillCheck => "skillCheck",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "start" => NodeType::Start,
            "end" => NodeType::End,
            "text" => NodeType::Text,
            "branchContainer" => NodeType::BranchContainer,
            "branch" => NodeType::Branch,
            "skillCheck" => NodeType::SkillCheck,
            _ => return None,
        })
    }

    /// Branch and skill check nodes live inside a branch container.
    pub fn is_child(self) -> bool {
        matches!(self, NodeType::Branch | NodeType::SkillCheck)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartData {
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextData {
    /// Speaker name.
    pub speaker: Option<String>,
    /// Portrait `res://` path.
    pub portrait: Option<String>,
    pub text: String,
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BranchContainerData {
    pub branches: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BranchData {
    pub text: String,
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillCheckData {
    pub text: String,
    /// Skill name.
    pub skill: Option<String>,
    pub difficulty: i64,
    pub next: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeData {
    Start(StartData),
    End,
    Text(TextData),
    BranchContainer(BranchContainerData),
    Branch(BranchData),
    SkillCheck(SkillCheckData),
}

impl NodeData {
    pub fn default_for(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Start => NodeData::Start(StartData::default()),
            NodeType::End => NodeData::End,
            NodeType::Text => NodeData::Text(TextData::default()),
            NodeType::BranchContainer => NodeData::BranchContainer(BranchContainerData::default()),
            NodeType::Branch => NodeData::Branch(BranchData::default()),
            NodeType::SkillCheck => NodeData::SkillCheck(SkillCheckData {
                difficulty: 1,
                ..Default::default()
            }),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self {
            NodeData::Start(_) => NodeType::Start,
            NodeData::End => NodeType::End,
            NodeData::Text(_) => NodeType::Text,
            NodeData::BranchContainer(_) => NodeType::BranchContainer,
            NodeData::Branch(_) => NodeType::Branch,
            NodeData::SkillCheck(_) => NodeType::SkillCheck,
        }
    }

    pub fn next(&self) -> Option<&str> {
        match self {
            NodeData::Start(d) => d.next.as_deref(),
            NodeData::Text(d) => d.next.as_deref(),
            NodeData::Branch(d) => d.next.as_deref(),
            NodeData::SkillCheck(d) => d.next.as_deref(),
            NodeData::End | NodeData::BranchContainer(_) => None,
        }
    }

    /// The `next` pointer of a connectable node, or `None` for nodes that
    /// cannot be a connection source.
    fn next_mut(&mut self) -> Option<&mut Option<String>> {
        match self {
            NodeData::Start(d) => Some(&mut d.next),
            NodeData::Text(d) => Some(&mut d.next),
            NodeData::Branch(d) => Some(&mut d.next),
            NodeData::SkillCheck(d) => Some(&mut d.next),
            NodeData::End | NodeData::BranchContainer(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub id: String,
    pub parent_id: Option<String>,
    /// Canvas position. Child nodes are laid out by their container.
    pub position: Pos2,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub data: NodeData,
}

impl GraphNode {
    pub fn node_type(&self) -> NodeType {
        self.data.node_type()
    }
}

/// A drawn connection. The source node's `next` is the source of truth; the
/// edge also remembers which handles it uses.
#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Adds a top-level node and returns its id.
    pub fn add_node(&mut self, node_type: NodeType, position: Pos2) -> String {
        debug_assert!(!node_type.is_child());
        let id = new_id();
        let (width, height) = if node_type == NodeType::BranchContainer {
            (
                Some(BRANCH_CONTAINER_MIN_WIDTH),
                Some(BRANCH_CONTAINER_MIN_HEIGHT),
            )
        } else {
            (None, None)
        };
        self.nodes.push(GraphNode {
            id: id.clone(),
            parent_id: None,
            position,
            width,
            height,
            data: NodeData::default_for(node_type),
        });
        id
    }

    /// Adds a branch or skill check node to a branch container.
    pub fn add_child(
        &mut self,
        container_id: &str,
        node_type: NodeType,
        default_skill: Option<String>,
    ) -> Option<String> {
        debug_assert!(node_type.is_child());
        let id = new_id();
        let container = self.node_mut(container_id)?;
        let NodeData::BranchContainer(data) = &mut container.data else {
            return None;
        };
        data.branches.push(id.clone());

        let mut child_data = NodeData::default_for(node_type);
        if let NodeData::SkillCheck(d) = &mut child_data {
            d.skill = default_skill;
        }
        self.nodes.push(GraphNode {
            id: id.clone(),
            parent_id: Some(container_id.to_string()),
            position: pos2(0.0, 0.0),
            width: None,
            height: None,
            data: child_data,
        });
        Some(id)
    }

    /// Ids of a container's children in display order. Falls back to
    /// `parent_id` for children missing from the container's branch list.
    pub fn children_of(&self, container_id: &str) -> Vec<String> {
        let mut ids: Vec<String> = match self.node(container_id).map(|n| &n.data) {
            Some(NodeData::BranchContainer(d)) => d
                .branches
                .iter()
                .filter(|b| self.node(b).is_some())
                .cloned()
                .collect(),
            _ => Vec::new(),
        };
        for node in &self.nodes {
            if node.parent_id.as_deref() == Some(container_id) && !ids.contains(&node.id) {
                ids.push(node.id.clone());
            }
        }
        ids
    }

    /// Removes a node together with its children, every edge touching it, and
    /// any `next` pointers that referenced it.
    pub fn remove_node(&mut self, id: &str) {
        let mut removed = vec![id.to_string()];
        removed.extend(self.children_of(id));

        let parent_id = self.node(id).and_then(|n| n.parent_id.clone());
        if let Some(parent_id) = parent_id
            && let Some(NodeData::BranchContainer(d)) =
                self.node_mut(&parent_id).map(|n| &mut n.data)
        {
            d.branches.retain(|b| b != id);
        }

        self.nodes.retain(|n| !removed.contains(&n.id));
        self.edges
            .retain(|e| !removed.contains(&e.source) && !removed.contains(&e.target));
        for node in &mut self.nodes {
            if let Some(next) = node.data.next_mut()
                && next.as_ref().is_some_and(|n| removed.contains(n))
            {
                *next = None;
            }
        }
    }

    /// Connects `source` to `target`, replacing the source's previous
    /// outgoing edge since every node has a single `next`.
    pub fn connect(
        &mut self,
        source: &str,
        source_handle: Option<String>,
        target: &str,
        target_handle: Option<String>,
    ) -> bool {
        if source == target || self.node(target).is_none() {
            return false;
        }
        let Some(next) = self.node_mut(source).and_then(|n| n.data.next_mut()) else {
            return false;
        };
        *next = Some(target.to_string());

        self.edges.retain(|e| e.source != source);
        self.edges.push(Edge {
            id: new_id(),
            source: source.to_string(),
            target: target.to_string(),
            source_handle,
            target_handle,
        });
        true
    }

    pub fn disconnect(&mut self, edge_id: &str) {
        let Some(index) = self.edges.iter().position(|e| e.id == edge_id) else {
            return;
        };
        let edge = self.edges.remove(index);
        if let Some(next) = self.node_mut(&edge.source).and_then(|n| n.data.next_mut()) {
            *next = None;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TreeItemKind {
    Folder,
    Dialogue(Graph),
}

/// A folder or dialogue file in the repository directory.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem {
    pub id: String,
    /// Folder name, or the dialogue's file name without `.json`.
    pub name: String,
    /// `ROOT_ID` for top-level items.
    pub parent: String,
    pub kind: TreeItemKind,
}

impl TreeItem {
    pub fn is_folder(&self) -> bool {
        matches!(self.kind, TreeItemKind::Folder)
    }

    /// The name on disk.
    pub fn file_name(&self) -> String {
        if self.is_folder() {
            self.name.clone()
        } else {
            format!("{}.json", self.name)
        }
    }
}

/// The folder/dialogue hierarchy shown in the sidebar, mirroring the
/// repository directory.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DialogueTree {
    pub items: Vec<TreeItem>,
}

impl DialogueTree {
    pub fn get(&self, id: &str) -> Option<&TreeItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut TreeItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Children sorted like a file browser: folders first, then by name.
    pub fn children(&self, parent: &str) -> Vec<&TreeItem> {
        let mut children: Vec<&TreeItem> =
            self.items.iter().filter(|i| i.parent == parent).collect();
        children.sort_by_cached_key(|i| (!i.is_folder(), i.name.to_lowercase()));
        children
    }

    pub fn graph(&self, id: &str) -> Option<&Graph> {
        match &self.get(id)?.kind {
            TreeItemKind::Dialogue(g) => Some(g),
            TreeItemKind::Folder => None,
        }
    }

    pub fn graph_mut(&mut self, id: &str) -> Option<&mut Graph> {
        match &mut self.get_mut(id)?.kind {
            TreeItemKind::Dialogue(g) => Some(g),
            TreeItemKind::Folder => None,
        }
    }

    pub fn insert(&mut self, parent: &str, name: String, kind: TreeItemKind) -> String {
        let id = new_id();
        self.items.push(TreeItem {
            id: id.clone(),
            name,
            parent: parent.to_string(),
            kind,
        });
        id
    }

    /// The item's path relative to the repository root.
    pub fn rel_path(&self, id: &str) -> Option<std::path::PathBuf> {
        let mut parts = Vec::new();
        let mut current = self.get(id)?;
        parts.push(current.file_name());
        while current.parent != ROOT_ID {
            current = self.get(&current.parent)?;
            parts.push(current.file_name());
        }
        Some(parts.iter().rev().collect())
    }

    /// Whether a sibling (other than `except`) already uses this file name.
    /// Case-insensitive, since Windows and macOS file systems usually are.
    pub fn name_taken(&self, parent: &str, file_name: &str, except: Option<&str>) -> bool {
        self.children(parent)
            .iter()
            .any(|i| Some(i.id.as_str()) != except && i.file_name().eq_ignore_ascii_case(file_name))
    }

    /// `base`, or `base 2`, `base 3`, … — whichever is free in `parent`.
    pub fn unique_name(&self, parent: &str, base: &str, folder: bool) -> String {
        let file_name = |n: &str| {
            if folder {
                n.to_string()
            } else {
                format!("{n}.json")
            }
        };
        let mut name = base.to_string();
        let mut i = 2;
        while self.name_taken(parent, &file_name(&name), None) {
            name = format!("{base} {i}");
            i += 1;
        }
        name
    }

    /// Removes an item and everything below it.
    pub fn remove(&mut self, id: &str) {
        let doomed: Vec<String> = self
            .items
            .iter()
            .filter(|i| i.id == id || self.is_descendant(&i.id, id))
            .map(|i| i.id.clone())
            .collect();
        self.items.retain(|i| !doomed.contains(&i.id));
    }

    /// Whether `id` sits somewhere below `ancestor`.
    pub fn is_descendant(&self, id: &str, ancestor: &str) -> bool {
        let mut current = self.get(id).map(|i| i.parent.as_str());
        while let Some(parent) = current {
            if parent == ancestor {
                return true;
            }
            current = self.get(parent).map(|i| i.parent.as_str());
        }
        false
    }

    /// Whether `id` may move into `folder`: the target must be a folder (or
    /// the root) and not the item itself or one of its descendants.
    pub fn can_move(&self, id: &str, folder: &str) -> bool {
        if id == folder || self.is_descendant(folder, id) || self.get(id).is_none() {
            return false;
        }
        folder == ROOT_ID || self.get(folder).is_some_and(TreeItem::is_folder)
    }

    pub fn move_item(&mut self, id: &str, folder: &str) -> bool {
        if !self.can_move(id, folder) {
            return false;
        }
        self.get_mut(id).expect("checked").parent = folder.to_string();
        true
    }

    /// Ids of every dialogue, in sidebar order.
    #[cfg(any(test, debug_assertions))]
    pub fn dialogues(&self) -> Vec<String> {
        fn visit(tree: &DialogueTree, parent: &str, out: &mut Vec<String>) {
            for child in tree.children(parent) {
                if child.is_folder() {
                    visit(tree, &child.id, out);
                } else {
                    out.push(child.id.clone());
                }
            }
        }
        let mut out = Vec::new();
        visit(self, ROOT_ID, &mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn remove_node_cleans_up_edges_and_next() {
        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 0.0));
        let text = g.add_node(NodeType::Text, pos2(100.0, 0.0));
        assert!(g.connect(&start, None, &text, Some("left-target".into())));
        g.remove_node(&text);
        assert!(g.edges.is_empty());
        assert_eq!(
            g.node(&start).unwrap().data,
            NodeData::Start(StartData { next: None })
        );
    }

    #[test]
    fn removing_container_removes_children() {
        let mut g = Graph::default();
        let container = g.add_node(NodeType::BranchContainer, pos2(0.0, 0.0));
        let branch = g.add_child(&container, NodeType::Branch, None).unwrap();
        let end = g.add_node(NodeType::End, pos2(300.0, 0.0));
        g.connect(&branch, None, &end, None);
        g.remove_node(&container);
        assert_eq!(g.nodes.len(), 1);
        assert!(g.edges.is_empty());
    }

    #[test]
    fn removing_child_updates_container() {
        let mut g = Graph::default();
        let container = g.add_node(NodeType::BranchContainer, pos2(0.0, 0.0));
        let a = g.add_child(&container, NodeType::Branch, None).unwrap();
        let b = g.add_child(&container, NodeType::Branch, None).unwrap();
        g.remove_node(&a);
        assert_eq!(g.children_of(&container), vec![b]);
    }

    #[test]
    fn connect_replaces_previous_edge() {
        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 0.0));
        let a = g.add_node(NodeType::Text, pos2(0.0, 0.0));
        let b = g.add_node(NodeType::End, pos2(0.0, 0.0));
        g.connect(&start, None, &a, None);
        g.connect(&start, None, &b, None);
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].target, b);
        assert!(!g.connect(&b, None, &a, None), "end nodes have no output");
        assert!(
            !g.connect(&a, None, &a, None),
            "self connections are rejected"
        );
    }

    #[test]
    fn tree_paths_and_names() {
        let mut t = DialogueTree::default();
        let outer = t.insert(ROOT_ID, "Chapter 1".into(), TreeItemKind::Folder);
        let inner = t.insert(&outer, "Scene".into(), TreeItemKind::Folder);
        let d = t.insert(
            &inner,
            "Intro".into(),
            TreeItemKind::Dialogue(Graph::default()),
        );
        assert_eq!(
            t.rel_path(&d).unwrap(),
            PathBuf::from("Chapter 1").join("Scene").join("Intro.json")
        );
        assert!(t.name_taken(&inner, "intro.JSON", None));
        assert!(!t.name_taken(&inner, "Intro.json", Some(&d)));
        assert_eq!(t.unique_name(&inner, "Intro", false), "Intro 2");
        assert_eq!(
            t.unique_name(&inner, "Intro", true),
            "Intro",
            "folders don't clash with files"
        );
    }

    #[test]
    fn folders_cannot_move_into_themselves() {
        let mut t = DialogueTree::default();
        let outer = t.insert(ROOT_ID, "a".into(), TreeItemKind::Folder);
        let inner = t.insert(&outer, "b".into(), TreeItemKind::Folder);
        assert!(!t.move_item(&outer, &inner));
        assert!(!t.move_item(&outer, &outer));
        assert!(t.move_item(&inner, ROOT_ID));
        t.remove(&outer);
        assert_eq!(t.items.len(), 1);
    }

    #[test]
    fn children_sort_folders_first() {
        let mut t = DialogueTree::default();
        t.insert(
            ROOT_ID,
            "b".into(),
            TreeItemKind::Dialogue(Graph::default()),
        );
        t.insert(ROOT_ID, "z".into(), TreeItemKind::Folder);
        t.insert(
            ROOT_ID,
            "A".into(),
            TreeItemKind::Dialogue(Graph::default()),
        );
        let names: Vec<&str> = t
            .children(ROOT_ID)
            .iter()
            .map(|i| i.name.as_str())
            .collect();
        assert_eq!(names, ["z", "A", "b"]);
    }
}

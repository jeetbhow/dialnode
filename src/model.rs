//! Core data types. Field names and JSON shapes match the original Electron
//! version so existing repositories keep working.

use eframe::egui::{Pos2, pos2};
use serde::{Deserialize, Serialize};

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const FORMAT_VERSION: &str = "1.0.0";

pub const ROOT_ID: &str = "root";
pub const EDGE_TYPE: &str = "dialogueEdge";
pub const BRANCH_CONTAINER_MIN_WIDTH: f32 = 240.0;
pub const BRANCH_CONTAINER_MIN_HEIGHT: f32 = 240.0;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn speaker_kind() -> String {
    "speaker".into()
}

fn portrait_kind() -> String {
    "portrait".into()
}

fn skill_kind() -> String {
    "skill".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub location: String,
    pub godot_project_location: Option<String>,
    pub created_on: String,
    pub app_version: String,
    pub format_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Speaker {
    pub id: String,
    #[serde(default = "speaker_kind")]
    pub kind: String,
    pub name: String,
}

impl Speaker {
    pub fn new(name: String) -> Self {
        Self {
            id: new_id(),
            kind: speaker_kind(),
            name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Portrait {
    pub id: String,
    #[serde(default = "portrait_kind")]
    pub kind: String,
    pub name: String,
    #[serde(rename = "dataURL")]
    pub data_url: String,
    pub width: u32,
    pub height: u32,
    pub path: String,
    pub rel_path: String,
    pub virtual_path: String,
    pub filename: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    #[serde(default = "skill_kind")]
    pub kind: String,
    pub category_id: String,
    pub name: String,
}

impl Skill {
    pub fn new(category_id: String, name: String) -> Self {
        Self {
            id: new_id(),
            kind: skill_kind(),
            category_id,
            name,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillCategory {
    pub id: String,
    pub name: String,
    pub skills: Vec<Skill>,
}

/// Everything stored in the repository database besides dialogues.
#[derive(Default)]
pub struct Entities {
    pub portraits: Vec<Portrait>,
    pub speakers: Vec<Speaker>,
    pub skill_categories: Vec<SkillCategory>,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StartData {
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TextData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speaker: Option<Speaker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait: Option<Portrait>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BranchContainerData {
    #[serde(default)]
    pub branches: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BranchData {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub next: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SkillCheckData {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub skill: Option<Skill>,
    #[serde(default)]
    pub difficulty: i64,
    #[serde(default)]
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

    pub fn to_json(&self) -> String {
        let result = match self {
            NodeData::Start(d) => serde_json::to_string(d),
            NodeData::End => Ok("{}".to_string()),
            NodeData::Text(d) => serde_json::to_string(d),
            NodeData::BranchContainer(d) => serde_json::to_string(d),
            NodeData::Branch(d) => serde_json::to_string(d),
            NodeData::SkillCheck(d) => serde_json::to_string(d),
        };
        result.expect("node data always serializes")
    }

    pub fn from_json(node_type: NodeType, json: &str) -> serde_json::Result<Self> {
        Ok(match node_type {
            NodeType::Start => NodeData::Start(serde_json::from_str(json)?),
            NodeType::End => NodeData::End,
            NodeType::Text => NodeData::Text(serde_json::from_str(json)?),
            NodeType::BranchContainer => NodeData::BranchContainer(serde_json::from_str(json)?),
            NodeType::Branch => NodeData::Branch(serde_json::from_str(json)?),
            NodeType::SkillCheck => NodeData::SkillCheck(serde_json::from_str(json)?),
        })
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
    pub extent: Option<String>,
    /// Absolute for top-level nodes, relative to the parent for child nodes.
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

#[derive(Clone, Debug, PartialEq)]
pub struct Edge {
    pub id: String,
    pub kind: String,
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
            extent: None,
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
        default_skill: Option<Skill>,
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
            extent: Some("parent".into()),
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
            kind: EDGE_TYPE.into(),
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

#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem {
    pub id: String,
    pub name: String,
    /// `ROOT_ID` for top-level items.
    pub parent: String,
    pub kind: TreeItemKind,
}

impl TreeItem {
    pub fn is_folder(&self) -> bool {
        matches!(self.kind, TreeItemKind::Folder)
    }
}

/// The folder/dialogue hierarchy shown in the sidebar. Sibling order is the
/// order of items in `items`.
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

    pub fn children(&self, parent: &str) -> Vec<&TreeItem> {
        self.items.iter().filter(|i| i.parent == parent).collect()
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

    pub fn add(&mut self, parent: &str, kind: TreeItemKind) -> String {
        let id = new_id();
        self.items.push(TreeItem {
            id: id.clone(),
            name: "Untitled".into(),
            parent: parent.to_string(),
            kind,
        });
        id
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

    /// Moves an item to the end of `folder`'s children. Refuses moves that
    /// would put a folder inside itself.
    pub fn move_item(&mut self, id: &str, folder: &str) -> bool {
        if id == folder || self.is_descendant(folder, id) {
            return false;
        }
        if folder != ROOT_ID && !self.get(folder).is_some_and(TreeItem::is_folder) {
            return false;
        }
        let Some(index) = self.items.iter().position(|i| i.id == id) else {
            return false;
        };
        let mut item = self.items.remove(index);
        item.parent = folder.to_string();
        self.items.push(item);
        true
    }

    /// Items in depth-first pre-order, the order used when saving.
    pub fn preorder(&self) -> Vec<&TreeItem> {
        fn visit<'a>(tree: &'a DialogueTree, parent: &str, out: &mut Vec<&'a TreeItem>) {
            for child in tree.children(parent) {
                out.push(child);
                visit(tree, &child.id, out);
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
    fn folders_cannot_move_into_themselves() {
        let mut t = DialogueTree::default();
        let outer = t.add(ROOT_ID, TreeItemKind::Folder);
        let inner = t.add(&outer, TreeItemKind::Folder);
        assert!(!t.move_item(&outer, &inner));
        assert!(!t.move_item(&outer, &outer));
        assert!(t.move_item(&inner, ROOT_ID));
        t.remove(&outer);
        assert_eq!(t.items.len(), 1);
    }

    #[test]
    fn node_data_json_round_trips_original_format() {
        let json = r#"{"speaker":{"id":"s","kind":"speaker","name":"Jeet"},"portrait":null,"text":"Hi","next":null}"#;
        let data = NodeData::from_json(NodeType::Text, json).unwrap();
        let NodeData::Text(t) = &data else { panic!() };
        assert_eq!(t.speaker.as_ref().unwrap().name, "Jeet");
        assert!(t.portrait.is_none());
        let again = NodeData::from_json(NodeType::Text, &data.to_json()).unwrap();
        assert_eq!(again, data);
    }
}

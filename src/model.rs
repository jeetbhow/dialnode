//! Core data types. Node data mirrors the dialogue file format.

use eframe::egui::Pos2;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ROOT_ID: &str = "root";
pub const BRANCH_CONTAINER_MIN_WIDTH: f32 = 240.0;
pub const BRANCH_CONTAINER_MIN_HEIGHT: f32 = 240.0;

pub fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
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
    /// Branch and skill check nodes live inside a branch container.
    pub fn is_child(self) -> bool {
        matches!(self, NodeType::Branch | NodeType::SkillCheck)
    }
}

/// A node's single outgoing connection, and which handles it's drawn
/// between. The target may be missing, in which case it isn't drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub target: String,
    pub source_handle: Option<String>,
    pub target_handle: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StartData {
    pub next: Option<Link>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextData {
    pub speaker: Option<String>,
    pub text: String,
    pub next: Option<Link>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BranchContainerData {
    /// Branches and skill checks, in display order.
    pub children: Vec<ChildNode>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BranchData {
    pub text: String,
    pub next: Option<Link>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillCheckData {
    pub text: String,
    /// Skill name.
    pub skill: Option<String>,
    pub difficulty: i64,
    pub next: Option<Link>,
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

    pub fn next(&self) -> Option<&Link> {
        match self {
            NodeData::Start(d) => d.next.as_ref(),
            NodeData::Text(d) => d.next.as_ref(),
            NodeData::Branch(d) => d.next.as_ref(),
            NodeData::SkillCheck(d) => d.next.as_ref(),
            NodeData::End | NodeData::BranchContainer(_) => None,
        }
    }

    /// The `next` pointer of a connectable node, or `None` for nodes that
    /// cannot be a connection source.
    fn next_mut(&mut self) -> Option<&mut Option<Link>> {
        match self {
            NodeData::Start(d) => Some(&mut d.next),
            NodeData::Text(d) => Some(&mut d.next),
            NodeData::Branch(d) => Some(&mut d.next),
            NodeData::SkillCheck(d) => Some(&mut d.next),
            NodeData::End | NodeData::BranchContainer(_) => None,
        }
    }
}

/// A branch or skill check. It lives inside a branch container, which lays
/// it out, so it has no position of its own.
#[derive(Clone, Debug, PartialEq)]
pub struct ChildNode {
    pub id: String,
    pub data: NodeData,
}

/// A top-level node on the canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub id: String,
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

/// A container's children, or none for any other node.
fn children(data: &NodeData) -> &[ChildNode] {
    match data {
        NodeData::BranchContainer(d) => &d.children,
        _ => &[],
    }
}

fn children_mut(data: &mut NodeData) -> Option<&mut Vec<ChildNode>> {
    match data {
        NodeData::BranchContainer(d) => Some(&mut d.children),
        _ => None,
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    /// Top-level nodes. Branches and skill checks are inside their
    /// containers.
    pub nodes: Vec<GraphNode>,
}

impl Graph {
    /// Every node, children included, as `(id, data)`.
    pub fn all_nodes(&self) -> impl Iterator<Item = (&str, &NodeData)> {
        self.nodes.iter().flat_map(|n| {
            std::iter::once((n.id.as_str(), &n.data))
                .chain(children(&n.data).iter().map(|c| (c.id.as_str(), &c.data)))
        })
    }

    fn for_each_data_mut(&mut self, mut f: impl FnMut(&mut NodeData)) {
        for node in &mut self.nodes {
            if let Some(children) = children_mut(&mut node.data) {
                for child in children {
                    f(&mut child.data);
                }
            }
            f(&mut node.data);
        }
    }

    /// The connections to draw, as `(source id, link)`: every link whose
    /// target exists.
    pub fn edges(&self) -> impl Iterator<Item = (&str, &Link)> {
        self.all_nodes().filter_map(|(id, data)| {
            let link = data.next()?;
            self.contains(&link.target).then_some((id, link))
        })
    }

    /// The node's link, if it's drawn.
    pub fn edge_from(&self, source: &str) -> Option<&Link> {
        let link = self.data(source)?.next()?;
        self.contains(&link.target).then_some(link)
    }

    /// Whether any node, top-level or child, has this id.
    pub fn contains(&self, id: &str) -> bool {
        self.data(id).is_some()
    }

    /// A top-level node.
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// A top-level node.
    pub fn node_mut(&mut self, id: &str) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    /// Any node's data, top-level or child.
    pub fn data(&self, id: &str) -> Option<&NodeData> {
        self.all_nodes().find(|(i, _)| *i == id).map(|(_, d)| d)
    }

    pub fn data_mut(&mut self, id: &str) -> Option<&mut NodeData> {
        for node in &mut self.nodes {
            if node.id == id {
                return Some(&mut node.data);
            }
            if let Some(children) = children_mut(&mut node.data)
                && let Some(child) = children.iter_mut().find(|c| c.id == id)
            {
                return Some(&mut child.data);
            }
        }
        None
    }

    /// The container a child node is in.
    pub fn parent_of(&self, id: &str) -> Option<&str> {
        self.nodes
            .iter()
            .find(|n| children(&n.data).iter().any(|c| c.id == id))
            .map(|n| n.id.as_str())
    }

    /// A container's children in display order.
    pub fn children_of(&self, container_id: &str) -> &[ChildNode] {
        self.node(container_id).map_or(&[], |n| children(&n.data))
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
            position,
            width,
            height,
            data: NodeData::default_for(node_type),
        });
        id
    }

    /// Adds a branch or skill check node to a branch container.
    pub fn add_child(&mut self, container_id: &str, node_type: NodeType) -> Option<String> {
        debug_assert!(node_type.is_child());
        let id = new_id();
        children_mut(&mut self.node_mut(container_id)?.data)?.push(ChildNode {
            id: id.clone(),
            data: NodeData::default_for(node_type),
        });
        Some(id)
    }

    /// Moves a branch or skill check to position `index` among its container's
    /// children (clamped to the last place). Returns whether the order changed.
    pub fn move_child(&mut self, child_id: &str, index: usize) -> bool {
        let Some(parent) = self.parent_of(child_id).map(String::from) else {
            return false;
        };
        let Some(children) = self
            .node_mut(&parent)
            .and_then(|n| children_mut(&mut n.data))
        else {
            return false;
        };
        let Some(from) = children.iter().position(|c| c.id == child_id) else {
            return false;
        };
        let to = index.min(children.len() - 1);
        if from == to {
            return false;
        }
        let child = children.remove(from);
        children.insert(to, child);
        true
    }

    /// Removes a node together with its children and any links to them.
    pub fn remove_node(&mut self, id: &str) {
        let mut removed = vec![id.to_string()];
        removed.extend(self.children_of(id).iter().map(|c| c.id.clone()));

        self.nodes.retain(|n| n.id != id);
        for node in &mut self.nodes {
            if let Some(children) = children_mut(&mut node.data) {
                children.retain(|c| c.id != id);
            }
        }
        self.for_each_data_mut(|data| {
            if let Some(next) = data.next_mut()
                && next.as_ref().is_some_and(|l| removed.contains(&l.target))
            {
                *next = None;
            }
        });
    }

    /// Connects `source` to `target`, replacing the source's previous link
    /// since every node has a single `next`.
    pub fn connect(
        &mut self,
        source: &str,
        source_handle: Option<String>,
        target: &str,
        target_handle: Option<String>,
    ) -> bool {
        if source == target || !self.contains(target) {
            return false;
        }
        let Some(next) = self.data_mut(source).and_then(NodeData::next_mut) else {
            return false;
        };
        *next = Some(Link {
            target: target.to_string(),
            source_handle,
            target_handle,
        });
        true
    }

    /// Removes `source`'s link.
    pub fn disconnect(&mut self, source: &str) {
        if let Some(next) = self.data_mut(source).and_then(NodeData::next_mut) {
            *next = None;
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TreeItemKind<D = Graph> {
    Folder,
    Dialogue(D),
}

/// A folder or dialogue file in the repository directory.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeItem<D = Graph> {
    pub id: String,
    /// Folder name, or the dialogue's file name without `.json`.
    pub name: String,
    /// `ROOT_ID` for top-level items.
    pub parent: String,
    pub kind: TreeItemKind<D>,
}

impl<D> TreeItem<D> {
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
pub struct DialogueTree<D = Graph> {
    pub items: Vec<TreeItem<D>>,
}

impl<D> DialogueTree<D> {
    pub fn get(&self, id: &str) -> Option<&TreeItem<D>> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut TreeItem<D>> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    /// Children sorted like a file browser: folders first, then by name.
    pub fn children(&self, parent: &str) -> Vec<&TreeItem<D>> {
        let mut children: Vec<&TreeItem<D>> =
            self.items.iter().filter(|i| i.parent == parent).collect();
        children.sort_by_cached_key(|i| (!i.is_folder(), i.name.to_lowercase()));
        children
    }

    pub fn dialogue(&self, id: &str) -> Option<&D> {
        match &self.get(id)?.kind {
            TreeItemKind::Dialogue(g) => Some(g),
            TreeItemKind::Folder => None,
        }
    }

    pub fn dialogue_mut(&mut self, id: &str) -> Option<&mut D> {
        match &mut self.get_mut(id)?.kind {
            TreeItemKind::Dialogue(g) => Some(g),
            TreeItemKind::Folder => None,
        }
    }

    /// The same tree, with each dialogue turned into an `E`.
    pub fn map<E>(self, mut f: impl FnMut(D) -> E) -> DialogueTree<E> {
        let items = self
            .items
            .into_iter()
            .map(|item| TreeItem {
                id: item.id,
                name: item.name,
                parent: item.parent,
                kind: match item.kind {
                    TreeItemKind::Folder => TreeItemKind::Folder,
                    TreeItemKind::Dialogue(d) => TreeItemKind::Dialogue(f(d)),
                },
            })
            .collect();
        DialogueTree { items }
    }

    pub fn insert(&mut self, parent: &str, name: String, kind: TreeItemKind<D>) -> String {
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
        folder == ROOT_ID || self.get(folder).is_some_and(|i| i.is_folder())
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
        fn visit<D>(tree: &DialogueTree<D>, parent: &str, out: &mut Vec<String>) {
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

    fn target(graph: &Graph, source: &str) -> Option<String> {
        graph.edge_from(source).map(|l| l.target.clone())
    }

    #[test]
    fn connect_replaces_the_previous_link() {
        let mut graph = Graph::default();
        let start = graph.add_node(NodeType::Start, Pos2::ZERO);
        let a = graph.add_node(NodeType::Text, Pos2::ZERO);
        let b = graph.add_node(NodeType::Text, Pos2::ZERO);
        assert!(graph.connect(&start, None, &a, Some("left-target".into())));
        assert!(graph.connect(&start, None, &b, None));
        assert_eq!(target(&graph, &start), Some(b.clone()));
        assert_eq!(graph.edges().count(), 1);
        assert_eq!(graph.edge_from(&start).unwrap().target_handle, None);
    }

    #[test]
    fn connect_refuses_self_links_and_nodes_without_next() {
        let mut graph = Graph::default();
        let a = graph.add_node(NodeType::Text, Pos2::ZERO);
        let end = graph.add_node(NodeType::End, Pos2::ZERO);
        assert!(!graph.connect(&a, None, &a, None));
        assert!(!graph.connect(&end, None, &a, None));
        assert!(!graph.connect(&a, None, "missing", None));
        assert_eq!(graph.edges().count(), 0);
    }

    #[test]
    fn disconnect_clears_the_link() {
        let mut graph = Graph::default();
        let a = graph.add_node(NodeType::Text, Pos2::ZERO);
        let end = graph.add_node(NodeType::End, Pos2::ZERO);
        graph.connect(&a, None, &end, None);
        graph.disconnect(&a);
        assert_eq!(graph.node(&a).unwrap().data.next(), None);
    }

    #[test]
    fn removing_a_container_removes_its_children_and_links_to_them() {
        let mut graph = Graph::default();
        let a = graph.add_node(NodeType::Text, Pos2::ZERO);
        let container = graph.add_node(NodeType::BranchContainer, Pos2::ZERO);
        let branch = graph.add_child(&container, NodeType::Branch).unwrap();
        let end = graph.add_node(NodeType::End, Pos2::ZERO);
        graph.connect(&a, None, &container, None);
        graph.connect(&branch, None, &end, None);

        graph.remove_node(&container);
        assert!(!graph.contains(&branch));
        assert_eq!(graph.node(&a).unwrap().data.next(), None);
        assert_eq!(graph.edges().count(), 0);
    }

    fn child_ids(graph: &Graph, container: &str) -> Vec<String> {
        graph
            .children_of(container)
            .iter()
            .map(|c| c.id.clone())
            .collect()
    }

    #[test]
    fn children_can_be_reordered_and_removed() {
        let mut graph = Graph::default();
        let container = graph.add_node(NodeType::BranchContainer, Pos2::ZERO);
        let first = graph.add_child(&container, NodeType::Branch).unwrap();
        let second = graph.add_child(&container, NodeType::SkillCheck).unwrap();
        assert_eq!(graph.parent_of(&second), Some(container.as_str()));
        assert!(graph.move_child(&second, 0));
        assert!(!graph.move_child(&second, 0));
        assert_eq!(
            child_ids(&graph, &container),
            [second.clone(), first.clone()]
        );
        graph.remove_node(&second);
        assert_eq!(child_ids(&graph, &container), [first]);
    }

    #[test]
    fn children_link_like_top_level_nodes() {
        let mut graph = Graph::default();
        let container = graph.add_node(NodeType::BranchContainer, Pos2::ZERO);
        let branch = graph.add_child(&container, NodeType::Branch).unwrap();
        let end = graph.add_node(NodeType::End, Pos2::ZERO);
        assert!(graph.connect(&branch, None, &end, None));
        assert_eq!(graph.edge_from(&branch).unwrap().target, end);
        graph.remove_node(&end);
        assert_eq!(graph.data(&branch).unwrap().next(), None);
    }
}

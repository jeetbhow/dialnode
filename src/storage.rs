//! On-disk format. A repository is any folder:
//!
//! ```text
//! Chapter 1/         folders are plain directories
//!   Intro.json       each dialogue is a JSON file
//! ```
//!
//! A dialogue file is the node list documented in the README. Each node can
//! carry an extra `layout` object (canvas position, container size, and which
//! handles its outgoing edge uses); games can ignore it.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use eframe::egui::pos2;
use serde::{Deserialize, Serialize};

use crate::model::{
    BranchContainerData, BranchData, ChildNode, DialogueTree, Graph, GraphNode, Link, NodeData,
    ROOT_ID, SkillCheckData, StartData, TextData, TreeItemKind,
};

const DIALOGUE_EXTENSION: &str = "json";
/// Spacing for nodes in files that don't say where they go.
const AUTO_LAYOUT_STEP: f32 = 320.0;

/// One node as it's written in a dialogue file.
#[derive(Serialize, Deserialize)]
struct NodeRecord {
    id: String,
    #[serde(flatten)]
    body: NodeBody,
    #[serde(default, skip_serializing_if = "Layout::is_empty")]
    layout: Layout,
}

/// A node's `type` and the fields that go with it. Missing fields get their
/// defaults, so hand-written files can leave them out.
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum NodeBody {
    Start {
        next: Option<String>,
    },
    End,
    Text {
        #[serde(default)]
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        speaker: Option<String>,
        next: Option<String>,
    },
    BranchContainer {
        #[serde(default)]
        branches: Vec<String>,
    },
    Branch {
        parent_id: Option<String>,
        #[serde(default)]
        text: String,
        next: Option<String>,
    },
    SkillCheck {
        parent_id: Option<String>,
        #[serde(default)]
        text: String,
        skill: Option<String>,
        #[serde(default = "default_difficulty")]
        difficulty: i64,
        next: Option<String>,
    },
}

fn default_difficulty() -> i64 {
    1
}

/// Editor-only data: where the node sits and which handles its link uses.
#[derive(Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Layout {
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_handle: Option<String>,
}

impl Layout {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// The record for any node. `layout` holds its placement, if it has one;
/// the handles its link uses are added here.
fn to_record(id: &str, data: &NodeData, parent_id: Option<&str>, mut layout: Layout) -> NodeRecord {
    let link = data.next();
    layout.source_handle = link.and_then(|l| l.source_handle.clone());
    layout.target_handle = link.and_then(|l| l.target_handle.clone());
    let next = link.map(|l| l.target.clone());
    let parent_id = parent_id.map(String::from);
    let body = match data {
        NodeData::Start(_) => NodeBody::Start { next },
        NodeData::End => NodeBody::End,
        NodeData::Text(d) => NodeBody::Text {
            text: d.text.clone(),
            speaker: d.speaker.clone(),
            next,
        },
        NodeData::BranchContainer(d) => NodeBody::BranchContainer {
            branches: d.children.iter().map(|c| c.id.clone()).collect(),
        },
        NodeData::Branch(d) => NodeBody::Branch {
            parent_id,
            text: d.text.clone(),
            next,
        },
        NodeData::SkillCheck(d) => NodeBody::SkillCheck {
            parent_id,
            text: d.text.clone(),
            skill: d.skill.clone(),
            difficulty: d.difficulty,
            next,
        },
    };
    NodeRecord {
        id: id.to_string(),
        body,
        layout,
    }
}

/// Every node's record, each container followed by its children.
fn dialogue_records(graph: &Graph) -> Vec<NodeRecord> {
    let mut records = Vec::new();
    for node in &graph.nodes {
        // Children are positioned by their container, so only top-level
        // nodes record a position.
        let placement = Layout {
            x: Some(node.position.x),
            y: Some(node.position.y),
            width: node.width,
            height: node.height,
            ..Layout::default()
        };
        records.push(to_record(&node.id, &node.data, None, placement));
        for child in graph.children_of(&node.id) {
            records.push(to_record(
                &child.id,
                &child.data,
                Some(&node.id),
                Layout::default(),
            ));
        }
    }
    records
}

/// A node read from a file, before children are put in their containers.
struct LoadedNode {
    node: GraphNode,
    /// A branch or skill check's `parentId`.
    parent_id: Option<String>,
    /// A container's `branches`: the order of its children.
    branches: Vec<String>,
}

/// `index` is the node's place in the file, used to spread out nodes that
/// have no position.
fn from_record(index: usize, record: NodeRecord) -> LoadedNode {
    let NodeRecord { id, body, layout } = record;
    let link = |next: Option<String>| {
        next.map(|target| Link {
            target,
            source_handle: layout.source_handle.clone(),
            target_handle: layout.target_handle.clone(),
        })
    };
    let mut parent_id = None;
    let mut branches = Vec::new();
    let data = match body {
        NodeBody::Start { next } => NodeData::Start(StartData { next: link(next) }),
        NodeBody::End => NodeData::End,
        NodeBody::Text {
            text,
            speaker,
            next,
        } => NodeData::Text(TextData {
            speaker,
            text,
            next: link(next),
        }),
        NodeBody::BranchContainer { branches: order } => {
            branches = order;
            NodeData::BranchContainer(BranchContainerData::default())
        }
        NodeBody::Branch {
            parent_id: parent,
            text,
            next,
        } => {
            parent_id = parent;
            NodeData::Branch(BranchData {
                text,
                next: link(next),
            })
        }
        NodeBody::SkillCheck {
            parent_id: parent,
            text,
            skill,
            difficulty,
            next,
        } => {
            parent_id = parent;
            NodeData::SkillCheck(SkillCheckData {
                text,
                skill,
                difficulty,
                next: link(next),
            })
        }
    };
    let node = GraphNode {
        id,
        position: pos2(
            layout.x.unwrap_or(index as f32 * AUTO_LAYOUT_STEP),
            layout.y.unwrap_or(0.0),
        ),
        width: layout.width,
        height: layout.height,
        data,
    };
    LoadedNode {
        node,
        parent_id,
        branches,
    }
}

/// Reads a dialogue file. Links to nodes that don't exist are kept, and just
/// don't draw. A branch or skill check whose parent isn't a container in
/// the file stays on the canvas as a top-level node.
pub fn parse_dialogue(text: &str) -> Result<Graph> {
    let records: Vec<NodeRecord> = serde_json::from_str(text)?;
    let loaded: Vec<LoadedNode> = records
        .into_iter()
        .enumerate()
        .map(|(index, record)| from_record(index, record))
        .collect();
    let orders: HashMap<String, Vec<String>> = loaded
        .iter()
        .filter(|l| matches!(l.node.data, NodeData::BranchContainer(_)))
        .map(|l| (l.node.id.clone(), l.branches.clone()))
        .collect();

    let mut graph = Graph::default();
    let mut children = Vec::new();
    for l in loaded {
        match l.parent_id.filter(|p| orders.contains_key(p)) {
            Some(parent) => children.push((parent, l.node)),
            None => graph.nodes.push(l.node),
        }
    }
    // Children go in the order their container lists them, then any it
    // doesn't list in file order.
    children.sort_by_key(|(parent, child)| {
        orders[parent]
            .iter()
            .position(|b| *b == child.id)
            .unwrap_or(usize::MAX)
    });
    for (parent, child) in children {
        if let Some(NodeData::BranchContainer(d)) = graph.data_mut(&parent) {
            d.children.push(ChildNode {
                id: child.id,
                data: child.data,
            });
        }
    }
    Ok(graph)
}

/// Writes through a temporary file so a crash can't leave a half-written
/// dialogue behind.
fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to write {}", path.display()))
}

fn dialogue_text(graph: &Graph) -> Result<String> {
    let records = dialogue_records(graph);
    let mut text = serde_json::to_string_pretty(&records)?;
    text.push('\n');
    Ok(text)
}

pub fn write_dialogue(path: &Path, graph: &Graph) -> Result<()> {
    write_atomic(path, &dialogue_text(graph)?)
}

/// Reads the folder/dialogue tree under `root`. Hidden entries are skipped.
/// Files that aren't valid dialogues are left
/// out and reported in the returned warnings.
pub fn load_tree(root: &Path) -> Result<(DialogueTree, Vec<String>)> {
    let mut tree = DialogueTree::default();
    let mut warnings = Vec::new();
    load_dir(root, root, ROOT_ID, &mut tree, &mut warnings)?;
    Ok((tree, warnings))
}

fn load_dir(
    root: &Path,
    dir: &Path,
    parent: &str,
    tree: &mut DialogueTree,
    warnings: &mut Vec<String>,
) -> Result<()> {
    let entries = fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type()?.is_dir() {
            let id = tree.insert(parent, name, TreeItemKind::Folder);
            load_dir(root, &path, &id, tree, warnings)?;
            continue;
        }
        let is_dialogue = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case(DIALOGUE_EXTENSION));
        if !is_dialogue {
            continue;
        }
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        match fs::read_to_string(&path)
            .map_err(anyhow::Error::from)
            .and_then(|t| parse_dialogue(&t))
        {
            Ok(graph) => {
                tree.insert(parent, stem, TreeItemKind::Dialogue(graph));
            }
            Err(err) => warnings.push(format!("{rel}: {err:#}")),
        }
    }
    Ok(())
}

/// Checks a folder or dialogue name typed by the user. Returns it trimmed,
/// with a trailing `.json` removed for dialogues.
pub fn validate_name(name: &str, folder: bool) -> Result<String> {
    let mut name = name.trim();
    if !folder && name.len() > 5 && name[name.len() - 5..].eq_ignore_ascii_case(".json") {
        name = name[..name.len() - 5].trim_end();
    }
    if name.is_empty() {
        bail!("Names can't be empty.");
    }
    if name.starts_with('.') {
        bail!("Names can't start with a dot.");
    }
    if let Some(c) = name.chars().find(|c| {
        matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') || c.is_control()
    }) {
        bail!("Names can't contain {c:?}.");
    }
    Ok(name.to_string())
}

/// Creates an empty dialogue file or a folder.
pub fn create(path: &Path, folder: bool) -> Result<()> {
    if path.exists() {
        bail!("{} already exists.", path.display());
    }
    if folder {
        fs::create_dir(path).with_context(|| format!("failed to create {}", path.display()))
    } else {
        write_dialogue(path, &Graph::default())
    }
}

/// Renames or moves a file or folder, refusing to replace anything. The
/// existence check is skipped when only the letter case changes, since on
/// case-insensitive file systems the target "exists" already.
pub fn rename(from: &Path, to: &Path) -> Result<()> {
    let case_only = from
        .to_string_lossy()
        .eq_ignore_ascii_case(&to.to_string_lossy());
    if to.exists() && !case_only {
        bail!("{} already exists.", to.display());
    }
    fs::rename(from, to)
        .with_context(|| format!("failed to move {} to {}", from.display(), to.display()))
}

/// Moves a file or folder to the system trash.
pub fn trash(path: &Path) -> Result<()> {
    trash::delete(path).with_context(|| format!("couldn't move {} to the trash", path.display()))
}

pub fn path_of<D>(root: &Path, tree: &DialogueTree<D>, id: &str) -> Option<PathBuf> {
    tree.rel_path(id).map(|rel| root.join(rel))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every node type, with layouts and handles, in the exact form
    /// `write_dialogue` produces.
    const FULL: &str = r#"[
  {
    "id": "start",
    "type": "start",
    "next": "line",
    "layout": {
      "x": 0.0,
      "y": 0.0,
      "targetHandle": "top-target"
    }
  },
  {
    "id": "line",
    "type": "text",
    "text": "What's your favourite colour?",
    "speaker": "Jeet",
    "next": "choice",
    "layout": {
      "x": 300.0,
      "y": -20.5,
      "sourceHandle": "bottom-source"
    }
  },
  {
    "id": "choice",
    "type": "branchContainer",
    "branches": [
      "blue",
      "check"
    ],
    "layout": {
      "x": 700.0,
      "y": 0.0,
      "width": 260.0,
      "height": 300.0
    }
  },
  {
    "id": "blue",
    "type": "branch",
    "parentId": "choice",
    "text": "Blue",
    "next": "end"
  },
  {
    "id": "check",
    "type": "skillCheck",
    "parentId": "choice",
    "text": "Red",
    "skill": null,
    "difficulty": 12,
    "next": "end"
  },
  {
    "id": "aside",
    "type": "text",
    "text": "",
    "next": null,
    "layout": {
      "x": 0.0,
      "y": 400.0
    }
  },
  {
    "id": "end",
    "type": "end",
    "layout": {
      "x": 1100.0,
      "y": 0.0
    }
  }
]
"#;

    fn targets(graph: &Graph) -> Vec<(String, String)> {
        let mut links: Vec<(String, String)> = graph
            .edges()
            .map(|(source, link)| (source.to_string(), link.target.clone()))
            .collect();
        links.sort();
        links
    }

    #[test]
    fn round_trip_is_byte_identical() {
        let graph = parse_dialogue(FULL).unwrap();
        assert_eq!(dialogue_text(&graph).unwrap(), FULL);
    }

    #[test]
    fn parses_fields_and_links() {
        let graph = parse_dialogue(FULL).unwrap();
        let line = graph.node("line").unwrap();
        assert_eq!(line.position, pos2(300.0, -20.5));
        let NodeData::Text(d) = &line.data else {
            panic!("line is a text node");
        };
        assert_eq!(d.speaker.as_deref(), Some("Jeet"));
        let choice = graph.node("choice").unwrap();
        assert_eq!((choice.width, choice.height), (Some(260.0), Some(300.0)));
        assert_eq!(child_ids(&graph, "choice"), ["blue", "check"]);
        assert_eq!(
            targets(&graph),
            [
                ("blue".into(), "end".into()),
                ("check".into(), "end".into()),
                ("line".into(), "choice".into()),
                ("start".into(), "line".into()),
            ]
        );
    }

    #[test]
    fn fills_in_missing_fields() {
        let graph = parse_dialogue(
            r#"[
                {"id": "a", "type": "text"},
                {"id": "b", "type": "skillCheck", "parentId": "c"},
                {"id": "c", "type": "branchContainer"}
            ]"#,
        )
        .unwrap();
        let a = graph.node("a").unwrap();
        assert_eq!(a.data, NodeData::Text(TextData::default()));
        assert_eq!(a.position, pos2(0.0, 0.0));
        let Some(NodeData::SkillCheck(b)) = graph.data("b") else {
            panic!("b is a skill check");
        };
        assert_eq!(b.difficulty, 1);
        // Files without a layout get their top-level nodes spread out.
        assert_eq!(
            graph.node("c").unwrap().position,
            pos2(2.0 * AUTO_LAYOUT_STEP, 0.0)
        );
        assert_eq!(child_ids(&graph, "c"), ["b"]);
    }

    #[test]
    fn keeps_dangling_links_without_drawing_them() {
        let text = r#"[{"id": "a", "type": "start", "next": "gone"}]"#;
        let graph = parse_dialogue(text).unwrap();
        assert!(targets(&graph).is_empty());
        assert!(dialogue_text(&graph).unwrap().contains(r#""next": "gone""#));
    }

    #[test]
    fn rejects_invalid_files() {
        assert!(parse_dialogue("not json").is_err());
        assert!(parse_dialogue(r#"{"id": "a"}"#).is_err());
        assert!(parse_dialogue(r#"[{"type": "end"}]"#).is_err());
        assert!(parse_dialogue(r#"[{"id": "a", "type": "nope"}]"#).is_err());
        assert!(parse_dialogue(r#"[{"id": "a"}]"#).is_err());
    }

    fn child_ids(graph: &Graph, container: &str) -> Vec<String> {
        graph
            .children_of(container)
            .iter()
            .map(|c| c.id.clone())
            .collect()
    }

    #[test]
    fn orders_children_by_the_branch_list_then_the_file() {
        let graph = parse_dialogue(
            r#"[
                {"id": "x", "type": "branch", "parentId": "c"},
                {"id": "y", "type": "branch", "parentId": "c"},
                {"id": "c", "type": "branchContainer", "branches": ["y", "gone"]},
                {"id": "z", "type": "skillCheck", "parentId": "c"}
            ]"#,
        )
        .unwrap();
        assert_eq!(child_ids(&graph, "c"), ["y", "x", "z"]);
        assert_eq!(graph.nodes.len(), 1);
        // Saving writes the container's children right after it.
        let saved = parse_dialogue(&dialogue_text(&graph).unwrap()).unwrap();
        assert_eq!(saved, graph);
    }

    #[test]
    fn keeps_children_without_a_container_on_the_canvas() {
        let graph = parse_dialogue(
            r#"[
                {"id": "a", "type": "end"},
                {"id": "b", "type": "branch", "parentId": "a", "text": "Hi"}
            ]"#,
        )
        .unwrap();
        let b = graph.node("b").expect("b is top-level");
        assert_eq!(b.position, pos2(AUTO_LAYOUT_STEP, 0.0));
        assert_eq!(graph.parent_of("b"), None);
    }
}

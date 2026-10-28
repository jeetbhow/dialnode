//! On-disk format. A repository is a directory:
//!
//! ```text
//! manifest.dial      repository metadata
//! database.json      speakers, skills, portraits
//! Chapter 1/         folders are plain directories
//!   Intro.json       each dialogue is a JSON file
//! ```
//!
//! A dialogue file is the node list documented in the README. Each node can
//! carry an extra `layout` object (canvas position, container size, and which
//! handles its outgoing edge uses); games can ignore it.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use eframe::egui::pos2;
use serde_json::{Map, Value, json};

use crate::model::{
    BranchContainerData, BranchData, DialogueTree, Edge, Entities, Graph, GraphNode, NodeData,
    NodeType, ROOT_ID, SkillCheckData, StartData, TextData, TreeItemKind, new_id,
};

pub const DATABASE_FILE: &str = "database.json";
const DIALOGUE_EXTENSION: &str = "json";
/// Spacing for nodes in files that don't say where they go.
const AUTO_LAYOUT_STEP: f32 = 320.0;

/// Builds a dialogue's JSON. With `layout` false this is exactly the export
/// format from the README.
pub fn dialogue_json(graph: &Graph, layout: bool) -> Value {
    let nodes = graph
        .nodes
        .iter()
        .map(|node| {
            let mut obj = Map::new();
            obj.insert("id".into(), json!(node.id));
            obj.insert("type".into(), json!(node.node_type().as_str()));
            match &node.data {
                NodeData::Start(d) => {
                    obj.insert("next".into(), json!(d.next));
                }
                NodeData::End => {}
                NodeData::Text(d) => {
                    obj.insert("text".into(), json!(d.text));
                    if let Some(speaker) = &d.speaker {
                        obj.insert("speaker".into(), json!(speaker));
                    }
                    if let Some(portrait) = &d.portrait {
                        obj.insert("portrait".into(), json!(portrait));
                    }
                    obj.insert("next".into(), json!(d.next));
                }
                NodeData::BranchContainer(d) => {
                    obj.insert("branches".into(), json!(d.branches));
                }
                NodeData::Branch(d) => {
                    obj.insert("parentId".into(), json!(node.parent_id));
                    obj.insert("text".into(), json!(d.text));
                    obj.insert("next".into(), json!(d.next));
                }
                NodeData::SkillCheck(d) => {
                    obj.insert("parentId".into(), json!(node.parent_id));
                    obj.insert("text".into(), json!(d.text));
                    obj.insert("skill".into(), json!(d.skill));
                    obj.insert("difficulty".into(), json!(d.difficulty));
                    obj.insert("next".into(), json!(d.next));
                }
            }
            if layout {
                let layout = node_layout(graph, node);
                if !layout.is_empty() {
                    obj.insert("layout".into(), Value::Object(layout));
                }
            }
            Value::Object(obj)
        })
        .collect();
    Value::Array(nodes)
}

fn node_layout(graph: &Graph, node: &GraphNode) -> Map<String, Value> {
    let mut layout = Map::new();
    // Children are positioned by their container, so only top-level nodes
    // record a position.
    if node.parent_id.is_none() {
        layout.insert("x".into(), json!(node.position.x));
        layout.insert("y".into(), json!(node.position.y));
        if let Some(w) = node.width {
            layout.insert("width".into(), json!(w));
        }
        if let Some(h) = node.height {
            layout.insert("height".into(), json!(h));
        }
    }
    if let Some(edge) = graph.edges.iter().find(|e| e.source == node.id) {
        if let Some(h) = &edge.source_handle {
            layout.insert("sourceHandle".into(), json!(h));
        }
        if let Some(h) = &edge.target_handle {
            layout.insert("targetHandle".into(), json!(h));
        }
    }
    layout
}

pub fn parse_dialogue(text: &str) -> Result<Graph> {
    let value: Value = serde_json::from_str(text).context("not valid JSON")?;
    let Value::Array(items) = value else {
        bail!("expected a list of nodes");
    };

    let mut graph = Graph::default();
    let mut handles = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let obj = item
            .as_object()
            .with_context(|| format!("node {index} is not an object"))?;
        let id = str_field(obj, "id").with_context(|| format!("node {index} has no id"))?;
        let kind = str_field(obj, "type").unwrap_or_default();
        let node_type = NodeType::parse(&kind)
            .with_context(|| format!("node {id} has unknown type {kind:?}"))?;

        let next = str_field(obj, "next");
        let text = str_field(obj, "text").unwrap_or_default();
        let data = match node_type {
            NodeType::Start => NodeData::Start(StartData { next }),
            NodeType::End => NodeData::End,
            NodeType::Text => NodeData::Text(TextData {
                speaker: str_field(obj, "speaker"),
                portrait: str_field(obj, "portrait"),
                text,
                next,
            }),
            NodeType::BranchContainer => NodeData::BranchContainer(BranchContainerData {
                branches: obj
                    .get("branches")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .filter_map(|b| b.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            }),
            NodeType::Branch => NodeData::Branch(BranchData { text, next }),
            NodeType::SkillCheck => NodeData::SkillCheck(SkillCheckData {
                text,
                skill: str_field(obj, "skill"),
                difficulty: obj.get("difficulty").and_then(Value::as_i64).unwrap_or(1),
                next,
            }),
        };

        let layout = obj.get("layout").and_then(Value::as_object);
        let num = |key: &str| {
            layout
                .and_then(|l| l.get(key))
                .and_then(Value::as_f64)
                .map(|v| v as f32)
        };
        let parent_id = str_field(obj, "parentId");
        let fallback = if parent_id.is_some() {
            pos2(0.0, 0.0)
        } else {
            pos2(index as f32 * AUTO_LAYOUT_STEP, 0.0)
        };
        handles.push((
            id.clone(),
            layout.and_then(|l| str_field(l, "sourceHandle")),
            layout.and_then(|l| str_field(l, "targetHandle")),
        ));
        graph.nodes.push(GraphNode {
            id,
            parent_id,
            position: pos2(
                num("x").unwrap_or(fallback.x),
                num("y").unwrap_or(fallback.y),
            ),
            width: num("width"),
            height: num("height"),
            data,
        });
    }

    // Edges are derived from `next`; dangling references just don't draw.
    for (source, source_handle, target_handle) in handles {
        let Some(target) = graph
            .node(&source)
            .and_then(|n| n.data.next())
            .map(String::from)
        else {
            continue;
        };
        if graph.node(&target).is_some() {
            graph.edges.push(Edge {
                id: new_id(),
                source,
                target,
                source_handle,
                target_handle,
            });
        }
    }
    Ok(graph)
}

fn str_field(obj: &Map<String, Value>, key: &str) -> Option<String> {
    obj.get(key).and_then(Value::as_str).map(String::from)
}

/// Writes through a temporary file so a crash can't leave a half-written
/// dialogue behind.
fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, contents).with_context(|| format!("failed to write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("failed to write {}", path.display()))
}

pub fn write_dialogue(path: &Path, graph: &Graph) -> Result<()> {
    let mut text = serde_json::to_string_pretty(&dialogue_json(graph, true))?;
    text.push('\n');
    write_atomic(path, &text)
}

/// Reads the folder/dialogue tree under `root`. Hidden entries and
/// `database.json` are skipped. Files that aren't valid dialogues are left
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
        if !is_dialogue || (dir == root && name.eq_ignore_ascii_case(DATABASE_FILE)) {
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

pub fn load_entities(root: &Path) -> Result<Entities> {
    let path = root.join(DATABASE_FILE);
    if !path.exists() {
        return Ok(Entities::default());
    }
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("{DATABASE_FILE} is not valid"))
}

pub fn save_entities(root: &Path, entities: &Entities) -> Result<()> {
    let mut text = serde_json::to_string_pretty(entities)?;
    text.push('\n');
    write_atomic(&root.join(DATABASE_FILE), &text)
}

/// Checks a folder or dialogue name typed by the user. Returns it trimmed,
/// with a trailing `.json` removed for dialogues.
pub fn validate_name(name: &str, folder: bool, at_root: bool) -> Result<String> {
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
    if !folder && at_root && name.eq_ignore_ascii_case("database") {
        bail!("\"database\" is reserved at the top level.");
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

pub fn path_of(root: &Path, tree: &DialogueTree, id: &str) -> Option<PathBuf> {
    tree.rel_path(id).map(|rel| root.join(rel))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Speaker, TreeItemKind};

    fn sample() -> Graph {
        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 10.0));
        let text = g.add_node(NodeType::Text, pos2(200.0, 10.0));
        let container = g.add_node(NodeType::BranchContainer, pos2(500.0, 0.0));
        let blue = g.add_child(&container, NodeType::Branch, None).unwrap();
        g.add_child(&container, NodeType::SkillCheck, Some("Strength".into()));
        let end = g.add_node(NodeType::End, pos2(900.0, 0.0));
        g.connect(&start, None, &text, Some("left-target".into()));
        g.connect(&text, Some("bottom-source".into()), &container, None);
        g.connect(&blue, None, &end, None);
        if let NodeData::Text(d) = &mut g.node_mut(&text).unwrap().data {
            d.text = "What's your favourite color?".into();
            d.speaker = Some("Jeet".into());
            d.portrait = Some("res://portraits/jeet.png".into());
        }
        g
    }

    /// Edge ids are regenerated on load, so compare everything else.
    fn normalized(mut g: Graph) -> Graph {
        for e in &mut g.edges {
            e.id.clear();
        }
        g.edges.sort_by(|a, b| a.source.cmp(&b.source));
        g
    }

    #[test]
    fn dialogue_round_trips() {
        let g = sample();
        let text = serde_json::to_string(&dialogue_json(&g, true)).unwrap();
        assert_eq!(normalized(parse_dialogue(&text).unwrap()), normalized(g));
    }

    #[test]
    fn export_matches_documented_format() {
        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 0.0));
        let text = g.add_node(NodeType::Text, pos2(0.0, 0.0));
        g.connect(&start, None, &text, None);
        if let NodeData::Text(d) = &mut g.node_mut(&text).unwrap().data {
            d.text = "Hi".into();
            d.speaker = Some("Jeet".into());
        }
        let out = serde_json::to_string(&dialogue_json(&g, false)).unwrap();
        assert_eq!(
            out,
            format!(
                r#"[{{"id":"{start}","type":"start","next":"{text}"}},{{"id":"{text}","type":"text","text":"Hi","speaker":"Jeet","next":null}}]"#
            )
        );
    }

    #[test]
    fn files_without_layout_still_load() {
        let readme_example = r#"[
          {"id": "a", "type": "start", "next": "b"},
          {"id": "b", "type": "text", "text": "Hi", "speaker": "Jeet", "next": "c"},
          {"id": "c", "type": "end"}
        ]"#;
        let g = parse_dialogue(readme_example).unwrap();
        assert_eq!(g.nodes.len(), 3);
        assert_eq!(g.edges.len(), 2);
        assert_ne!(g.nodes[0].position, g.nodes[1].position, "auto-laid out");
    }

    #[test]
    fn bad_files_are_reported() {
        assert!(parse_dialogue("{}").is_err());
        assert!(parse_dialogue(r#"[{"id":"a","type":"bogus"}]"#).is_err());
        assert!(parse_dialogue(r#"[{"type":"end"}]"#).is_err());
    }

    #[test]
    fn tree_loads_from_directories() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("Chapter 1/Scene")).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        write_dialogue(&root.join("Chapter 1/Scene/Intro.json"), &sample()).unwrap();
        write_dialogue(&root.join("Top.json"), &Graph::default()).unwrap();
        fs::write(root.join("broken.json"), "nope").unwrap();
        fs::write(root.join("notes.txt"), "").unwrap();
        save_entities(root, &Entities::default()).unwrap();

        let (tree, warnings) = load_tree(root).unwrap();
        let mut paths: Vec<String> = tree
            .items
            .iter()
            .map(|i| tree.rel_path(&i.id).unwrap().display().to_string())
            .collect();
        paths.sort();
        assert_eq!(
            paths,
            [
                "Chapter 1",
                "Chapter 1/Scene",
                "Chapter 1/Scene/Intro.json",
                "Top.json"
            ]
        );
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].starts_with("broken.json"));
        let intro = tree
            .dialogues()
            .into_iter()
            .find(|id| tree.get(id).unwrap().name == "Intro");
        assert!(
            matches!(&tree.get(&intro.unwrap()).unwrap().kind, TreeItemKind::Dialogue(g) if g.nodes.len() == 6)
        );
    }

    #[test]
    fn entities_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(load_entities(tmp.path()).unwrap(), Entities::default());
        let mut e = Entities::default();
        e.speakers.push(Speaker::new("Jeet".into()));
        save_entities(tmp.path(), &e).unwrap();
        assert_eq!(load_entities(tmp.path()).unwrap(), e);
    }

    #[test]
    fn name_validation() {
        assert_eq!(
            validate_name(" Intro.JSON ", false, false).unwrap(),
            "Intro"
        );
        assert_eq!(validate_name("a.json", true, false).unwrap(), "a.json");
        assert!(validate_name("", false, false).is_err());
        assert!(validate_name("a/b", true, false).is_err());
        assert!(validate_name(".hidden", true, false).is_err());
        assert!(validate_name("database", false, true).is_err());
        assert!(validate_name("database", false, false).is_ok());
        assert!(validate_name("database", true, true).is_ok());
    }

    #[test]
    fn rename_refuses_to_overwrite() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a.json");
        let b = tmp.path().join("b.json");
        create(&a, false).unwrap();
        create(&b, false).unwrap();
        assert!(rename(&a, &b).is_err());
        assert!(create(&a, false).is_err());
        rename(&a, &tmp.path().join("c.json")).unwrap();
    }
}

//! SQLite storage. The schema is identical to the Electron version's, so a
//! `db.sqlite` written by either app can be read by the other.

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context as _, Result};
use eframe::egui::pos2;
use rusqlite::{Connection, params};

use crate::model::{
    DialogueTree, Edge, Graph, GraphNode, NodeData, NodeType, Portrait, ROOT_ID, Skill,
    SkillCategory, Speaker, TreeItem, TreeItemKind,
};

pub const DB_FILE_NAME: &str = "db.sqlite";

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS dialogueNodes (
  id TEXT PRIMARY KEY,
  type TEXT CHECK(type IN('folder', 'dialogue')),
  name TEXT,
  parentId TEXT REFERENCES dialogueNodes(id) ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE IF NOT EXISTS nodes (
  id TEXT PRIMARY KEY,
  parentId TEXT,
  extent TEXT,
  dialogueId TEXT,
  type TEXT,
  positionX REAL,
  positionY REAL,
  width REAL,
  height REAL,
  data TEXT,
  FOREIGN KEY (dialogueId) REFERENCES dialogueNodes(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS edges (
  id TEXT PRIMARY KEY,
  dialogueId TEXT,
  type TEXT,
  source TEXT,
  target TEXT,
  sourceHandle TEXT,
  targetHandle TEXT,
  FOREIGN KEY (dialogueId) REFERENCES dialogueNodes(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS portraits (
  id TEXT PRIMARY KEY,
  kind TEXT,
  name TEXT,
  dataURL TEXT,
  width INTEGER,
  height INTEGER,
  path TEXT,
  relPath TEXT,
  virtualPath TEXT,
  filename TEXT
);
CREATE TABLE IF NOT EXISTS speakers (
  id TEXT PRIMARY KEY,
  kind TEXT,
  name TEXT
);
CREATE TABLE IF NOT EXISTS skillCategories (
  id TEXT PRIMARY KEY,
  kind TEXT,
  name TEXT
);
CREATE TABLE IF NOT EXISTS skills (
  id TEXT PRIMARY KEY,
  kind TEXT,
  categoryId TEXT,
  name TEXT,
  FOREIGN KEY (categoryId) REFERENCES skillCategories(id) ON DELETE CASCADE
);
"#;

pub struct Database {
    conn: Connection,
}

impl Database {
    /// Opens (creating if needed) the database file at `path`.
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)
            .with_context(|| format!("failed to open database {}", path.display()))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    /// Replaces every stored folder and dialogue with the contents of `tree`.
    pub fn save_dialogues(&mut self, tree: &DialogueTree) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM dialogueNodes", [])?;
        {
            let mut insert_item = tx.prepare(
                "INSERT INTO dialogueNodes (id, type, name, parentId) VALUES (?1, ?2, ?3, ?4)",
            )?;
            let mut insert_node = tx.prepare(
                r#"INSERT INTO nodes (id, parentId, extent, dialogueId, "type", positionX, positionY, width, height, "data")
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
            )?;
            let mut insert_edge = tx.prepare(
                r#"INSERT INTO edges (id, dialogueId, "type", source, target, sourceHandle, targetHandle)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            )?;

            insert_item.execute(params![ROOT_ID, "folder", "root", Option::<String>::None])?;
            for item in tree.preorder() {
                let kind = if item.is_folder() {
                    "folder"
                } else {
                    "dialogue"
                };
                insert_item.execute(params![item.id, kind, item.name, item.parent])?;

                let TreeItemKind::Dialogue(graph) = &item.kind else {
                    continue;
                };
                for n in &graph.nodes {
                    insert_node.execute(params![
                        n.id,
                        n.parent_id,
                        n.extent,
                        item.id,
                        n.node_type().as_str(),
                        n.position.x as f64,
                        n.position.y as f64,
                        n.width.map(f64::from),
                        n.height.map(f64::from),
                        n.data.to_json(),
                    ])?;
                }
                for e in &graph.edges {
                    insert_edge.execute(params![
                        e.id,
                        item.id,
                        e.kind,
                        e.source,
                        e.target,
                        e.source_handle,
                        e.target_handle,
                    ])?;
                }
            }
        }
        tx.commit().context("saveDialogues failed")?;
        Ok(())
    }

    pub fn load_dialogues(&self) -> Result<DialogueTree> {
        let mut graphs: HashMap<String, Graph> = HashMap::new();

        let mut stmt = self.conn.prepare(
            "SELECT id, parentId, extent, dialogueId, type, positionX, positionY, width, height, data FROM nodes",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<String>>(4)?,
                r.get::<_, Option<f64>>(5)?,
                r.get::<_, Option<f64>>(6)?,
                r.get::<_, Option<f64>>(7)?,
                r.get::<_, Option<f64>>(8)?,
                r.get::<_, Option<String>>(9)?,
            ))
        })?;
        for row in rows {
            let (id, parent_id, extent, dialogue_id, kind, x, y, w, h, data) = row?;
            let Some(node_type) = kind.as_deref().and_then(NodeType::parse) else {
                eprintln!("skipping node {id}: unknown type {kind:?}");
                continue;
            };
            let data = match NodeData::from_json(node_type, data.as_deref().unwrap_or("{}")) {
                Ok(d) => d,
                Err(err) => {
                    eprintln!("node {id} has unreadable data ({err}); using defaults");
                    NodeData::default_for(node_type)
                }
            };
            graphs
                .entry(dialogue_id.unwrap_or_default())
                .or_default()
                .nodes
                .push(GraphNode {
                    id,
                    parent_id,
                    extent,
                    position: pos2(x.unwrap_or(0.0) as f32, y.unwrap_or(0.0) as f32),
                    width: w.map(|v| v as f32),
                    height: h.map(|v| v as f32),
                    data,
                });
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, dialogueId, type, source, target, sourceHandle, targetHandle FROM edges",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, Option<String>>(1)?,
                Edge {
                    id: r.get(0)?,
                    kind: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    source: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    target: r.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    source_handle: r.get(5)?,
                    target_handle: r.get(6)?,
                },
            ))
        })?;
        for row in rows {
            let (dialogue_id, edge) = row?;
            graphs
                .entry(dialogue_id.unwrap_or_default())
                .or_default()
                .edges
                .push(edge);
        }

        let mut stmt = self
            .conn
            .prepare("SELECT id, type, name, parentId FROM dialogueNodes")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, Option<String>>(3)?,
            ))
        })?;
        let mut tree = DialogueTree::default();
        for row in rows {
            let (id, kind, name, parent) = row?;
            if id == ROOT_ID {
                continue;
            }
            let kind = if kind.as_deref() == Some("dialogue") {
                TreeItemKind::Dialogue(graphs.remove(&id).unwrap_or_default())
            } else {
                TreeItemKind::Folder
            };
            tree.items.push(TreeItem {
                id,
                name: name.unwrap_or_default(),
                parent: parent.unwrap_or_else(|| ROOT_ID.into()),
                kind,
            });
        }

        // Re-home items whose parent no longer exists.
        let ids: Vec<String> = tree.items.iter().map(|i| i.id.clone()).collect();
        for item in &mut tree.items {
            if item.parent != ROOT_ID && !ids.contains(&item.parent) {
                item.parent = ROOT_ID.into();
            }
        }
        Ok(tree)
    }

    pub fn create_portrait(&self, p: &Portrait) -> Result<()> {
        self.conn.execute(
            "INSERT INTO portraits (id, name, kind, dataURL, width, height, path, relPath, virtualPath, filename)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                p.id,
                p.name,
                p.kind,
                p.data_url,
                p.width,
                p.height,
                p.path,
                p.rel_path,
                p.virtual_path,
                p.filename
            ],
        )?;
        Ok(())
    }

    pub fn all_portraits(&self) -> Result<Vec<Portrait>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, name, dataURL, width, height, path, relPath, virtualPath, filename FROM portraits",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Portrait {
                id: r.get(0)?,
                kind: r
                    .get::<_, Option<String>>(1)?
                    .unwrap_or_else(|| "portrait".into()),
                name: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                data_url: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                width: r.get::<_, Option<u32>>(4)?.unwrap_or_default(),
                height: r.get::<_, Option<u32>>(5)?.unwrap_or_default(),
                path: r.get::<_, Option<String>>(6)?.unwrap_or_default(),
                rel_path: r.get::<_, Option<String>>(7)?.unwrap_or_default(),
                virtual_path: r.get::<_, Option<String>>(8)?.unwrap_or_default(),
                filename: r.get::<_, Option<String>>(9)?.unwrap_or_default(),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn delete_portrait(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM portraits WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn create_speaker(&self, s: &Speaker) -> Result<()> {
        self.conn.execute(
            "INSERT INTO speakers (id, kind, name) VALUES (?1, ?2, ?3)",
            params![s.id, s.kind, s.name],
        )?;
        Ok(())
    }

    pub fn all_speakers(&self) -> Result<Vec<Speaker>> {
        let mut stmt = self.conn.prepare("SELECT id, kind, name FROM speakers")?;
        let rows = stmt.query_map([], |r| {
            Ok(Speaker {
                id: r.get(0)?,
                kind: r
                    .get::<_, Option<String>>(1)?
                    .unwrap_or_else(|| "speaker".into()),
                name: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn delete_speaker(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM speakers WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn create_skill_category(&self, id: &str, name: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO skillCategories (id, kind, name) VALUES (?1, 'skillCategory', ?2)",
            params![id, name],
        )?;
        Ok(())
    }

    pub fn delete_skill_category(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM skillCategories WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn create_skill(&self, s: &Skill) -> Result<()> {
        self.conn.execute(
            "INSERT INTO skills (id, kind, categoryId, name) VALUES (?1, ?2, ?3, ?4)",
            params![s.id, s.kind, s.category_id, s.name],
        )?;
        Ok(())
    }

    pub fn delete_skill(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM skills WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Skill categories with their skills attached.
    pub fn all_skill_categories(&self) -> Result<Vec<SkillCategory>> {
        let mut stmt = self.conn.prepare("SELECT id, name FROM skillCategories")?;
        let mut categories = stmt
            .query_map([], |r| {
                Ok(SkillCategory {
                    id: r.get(0)?,
                    name: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    skills: Vec::new(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut stmt = self
            .conn
            .prepare("SELECT id, kind, categoryId, name FROM skills")?;
        let skills = stmt
            .query_map([], |r| {
                Ok(Skill {
                    id: r.get(0)?,
                    kind: r
                        .get::<_, Option<String>>(1)?
                        .unwrap_or_else(|| "skill".into()),
                    category_id: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    name: r.get::<_, Option<String>>(3)?.unwrap_or_default(),
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        for skill in skills {
            if let Some(category) = categories.iter_mut().find(|c| c.id == skill.category_id) {
                category.skills.push(skill);
            }
        }
        Ok(categories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BranchData, NodeType, TextData};
    use eframe::egui::pos2;

    #[test]
    fn dialogues_round_trip() {
        let mut db = Database::open_in_memory().unwrap();
        let mut tree = DialogueTree::default();
        let folder = tree.add(ROOT_ID, TreeItemKind::Folder);
        let dialogue = tree.add(&folder, TreeItemKind::Dialogue(Graph::default()));
        tree.add(ROOT_ID, TreeItemKind::Dialogue(Graph::default()));

        let graph = tree.graph_mut(&dialogue).unwrap();
        let start = graph.add_node(NodeType::Start, pos2(1.0, 2.0));
        let text = graph.add_node(NodeType::Text, pos2(3.0, 4.0));
        let container = graph.add_node(NodeType::BranchContainer, pos2(5.0, 6.0));
        let branch = graph.add_child(&container, NodeType::Branch, None).unwrap();
        graph.connect(&start, None, &text, Some("left-target".into()));
        graph.connect(&text, Some("right-source".into()), &container, None);
        if let NodeData::Text(TextData { text, .. }) = &mut graph.node_mut(&text).unwrap().data {
            *text = "Hello".into();
        }
        if let NodeData::Branch(BranchData { text, .. }) =
            &mut graph.node_mut(&branch).unwrap().data
        {
            *text = "Blue".into();
        }

        db.save_dialogues(&tree).unwrap();
        // Saving twice must not duplicate rows.
        db.save_dialogues(&tree).unwrap();
        let loaded = db.load_dialogues().unwrap();
        assert_eq!(loaded, tree);
    }

    #[test]
    fn deleting_a_category_deletes_its_skills() {
        let db = Database::open_in_memory().unwrap();
        db.create_skill_category("c", "Body").unwrap();
        db.create_skill(&Skill::new("c".into(), "Strength".into()))
            .unwrap();
        assert_eq!(db.all_skill_categories().unwrap()[0].skills.len(), 1);
        db.delete_skill_category("c").unwrap();
        assert!(db.all_skill_categories().unwrap().is_empty());
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM skills", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }
}

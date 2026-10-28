//! Creating and opening repositories: a folder holding `manifest.dial` and
//! `db.sqlite`.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::db::{DB_FILE_NAME, Database};
use crate::model::{APP_VERSION, FORMAT_VERSION, Repository, new_id};

pub const MANIFEST_FILE_NAME: &str = "manifest.dial";
pub const DEFAULT_REPOSITORY_NAME: &str = ".dialnode";

pub struct OpenRepository {
    pub repository: Repository,
    pub manifest_path: PathBuf,
    pub db: Database,
}

pub fn is_godot_project(dir: &Path) -> bool {
    dir.join("project.godot").is_file()
}

/// Creates `<location>/<name>` with a fresh database and manifest.
pub fn create(
    name: &str,
    location: &Path,
    godot_project_location: Option<&Path>,
) -> Result<OpenRepository> {
    let name = name.trim();
    if name.is_empty() {
        bail!("The repository name cannot be empty.");
    }
    if name.contains(['/', '\\']) || name == "." || name == ".." {
        bail!("The repository name cannot contain path separators.");
    }

    let dir = location.join(name);
    fs::create_dir(&dir).with_context(|| format!("failed to create {}", dir.display()))?;

    let result = (|| -> Result<OpenRepository> {
        let db = Database::open(&dir.join(DB_FILE_NAME))?;
        let repository = Repository {
            id: new_id(),
            name: name.to_string(),
            location: location.to_string_lossy().into_owned(),
            godot_project_location: godot_project_location
                .map(|p| p.to_string_lossy().into_owned()),
            created_on: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            app_version: APP_VERSION.into(),
            format_version: FORMAT_VERSION.into(),
        };
        let manifest_path = dir.join(MANIFEST_FILE_NAME);
        save_manifest(&repository, &manifest_path)?;
        Ok(OpenRepository {
            repository,
            manifest_path,
            db,
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&dir);
    }
    result.context("Failed to create the repository")
}

/// Opens the repository described by a `manifest.dial` file. The database is
/// looked up next to the manifest first, so moved repositories still open.
pub fn open(manifest_path: &Path) -> Result<OpenRepository> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let repository: Repository =
        serde_json::from_str(&raw).context("the manifest is not a valid Dialnode manifest")?;

    let beside_manifest = manifest_path
        .parent()
        .map(|dir| dir.join(DB_FILE_NAME))
        .filter(|p| p.is_file());
    let recorded = Path::new(&repository.location)
        .join(&repository.name)
        .join(DB_FILE_NAME);
    let db_path = match beside_manifest {
        Some(path) => path,
        None if recorded.is_file() => recorded,
        None => bail!(
            "Could not find {DB_FILE_NAME} next to {}",
            manifest_path.display()
        ),
    };

    Ok(OpenRepository {
        db: Database::open(&db_path)?,
        repository,
        manifest_path: manifest_path.to_path_buf(),
    })
}

pub fn save_manifest(repository: &Repository, manifest_path: &Path) -> Result<()> {
    fs::write(manifest_path, serde_json::to_string(repository)?)
        .with_context(|| format!("failed to write {}", manifest_path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_open() {
        let tmp = tempfile::tempdir().unwrap();
        let created = create("repo", tmp.path(), None).unwrap();
        assert!(tmp.path().join("repo").join(DB_FILE_NAME).is_file());
        drop(created.db);

        let opened = open(&created.manifest_path).unwrap();
        assert_eq!(opened.repository, created.repository);

        assert!(create("repo", tmp.path(), None).is_err(), "already exists");
        assert!(create("a/b", tmp.path(), None).is_err());
    }

    /// Writes a sample repository for manual testing:
    /// `DIALNODE_SEED_DIR=/tmp cargo test seed_sample -- --ignored`
    #[test]
    #[ignore]
    fn seed_sample_repository() {
        use crate::model::*;
        use eframe::egui::pos2;

        let dir = PathBuf::from(std::env::var("DIALNODE_SEED_DIR").expect("set DIALNODE_SEED_DIR"));
        let _ = fs::remove_dir_all(dir.join("sample"));
        let mut repo = create("sample", &dir, None).unwrap();

        let jeet = Speaker::new("Jeet".into());
        repo.db.create_speaker(&jeet).unwrap();
        repo.db.create_skill_category("body", "Body").unwrap();
        let strength = Skill::new("body".into(), "Strength".into());
        repo.db.create_skill(&strength).unwrap();

        let mut tree = DialogueTree::default();
        let folder = tree.add(ROOT_ID, TreeItemKind::Folder);
        tree.get_mut(&folder).unwrap().name = "Chapter 1".into();
        let id = tree.add(&folder, TreeItemKind::Dialogue(Graph::default()));
        tree.get_mut(&id).unwrap().name = "Favourite colour".into();
        tree.add(ROOT_ID, TreeItemKind::Dialogue(Graph::default()));

        let g = tree.graph_mut(&id).unwrap();
        let start = g.add_node(NodeType::Start, pos2(0.0, 120.0));
        let ask = g.add_node(NodeType::Text, pos2(200.0, 80.0));
        let branches = g.add_node(NodeType::BranchContainer, pos2(560.0, 60.0));
        let blue = g.add_child(&branches, NodeType::Branch, None).unwrap();
        let lift = g
            .add_child(&branches, NodeType::SkillCheck, Some(strength.clone()))
            .unwrap();
        let reply = g.add_node(NodeType::Text, pos2(920.0, 60.0));
        let end = g.add_node(NodeType::End, pos2(1280.0, 140.0));
        if let NodeData::Text(d) = &mut g.node_mut(&ask).unwrap().data {
            d.text = "What's your favourite color?".into();
            d.speaker = Some(jeet.clone());
        }
        if let NodeData::Branch(d) = &mut g.node_mut(&blue).unwrap().data {
            d.text = "Blue".into();
        }
        if let NodeData::SkillCheck(d) = &mut g.node_mut(&lift).unwrap().data {
            d.text = "Lift the crate".into();
            d.difficulty = 12;
        }
        if let NodeData::Text(d) = &mut g.node_mut(&reply).unwrap().data {
            d.text = "Cool, I'm a big fan of blue as well.".into();
            d.speaker = Some(jeet);
        }
        g.connect(&start, None, &ask, Some("left-target".into()));
        g.connect(&ask, Some("right-source".into()), &branches, None);
        g.connect(&blue, None, &reply, Some("left-target".into()));
        g.connect(&reply, Some("right-source".into()), &end, None);
        repo.db.save_dialogues(&tree).unwrap();
        println!("{}", repo.manifest_path.display());
    }
}

//! Creating and opening repositories. A repository is a directory with a
//! `manifest.dial` marker; see `storage` for the rest of the layout.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};

use crate::model::{APP_VERSION, Entities, FORMAT_VERSION, Repository, new_id};
use crate::storage;

pub const MANIFEST_FILE_NAME: &str = "manifest.dial";
pub const DEFAULT_REPOSITORY_NAME: &str = ".dialnode";
/// Present in repositories from before dialogues were files.
const LEGACY_DATABASE: &str = "db.sqlite";

pub struct OpenRepository {
    pub repository: Repository,
    pub root: PathBuf,
}

pub fn is_godot_project(dir: &Path) -> bool {
    dir.join("project.godot").is_file()
}

/// Creates `<location>/<name>` with a manifest and an empty database.
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

    let root = location.join(name);
    fs::create_dir(&root).with_context(|| format!("failed to create {}", root.display()))?;

    let result = (|| -> Result<OpenRepository> {
        let repository = Repository {
            id: new_id(),
            name: name.to_string(),
            godot_project_location: godot_project_location
                .map(|p| p.to_string_lossy().into_owned()),
            created_on: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            app_version: APP_VERSION.into(),
            format_version: FORMAT_VERSION.into(),
        };
        save_manifest(&repository, &root)?;
        storage::save_entities(&root, &Entities::default())?;
        Ok(OpenRepository {
            repository,
            root: root.clone(),
        })
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&root);
    }
    result.context("Failed to create the repository")
}

/// Opens the repository in `root`.
pub fn open(root: &Path) -> Result<OpenRepository> {
    let manifest_path = root.join(MANIFEST_FILE_NAME);
    let legacy =
        root.join(LEGACY_DATABASE).is_file() && !root.join(storage::DATABASE_FILE).exists();
    if legacy {
        bail!(
            "This repository uses the old SQLite format (db.sqlite), which this version of \
             Dialnode no longer opens."
        );
    }
    if !manifest_path.is_file() {
        bail!(
            "{} is not a Dialnode repository (it has no {MANIFEST_FILE_NAME}).",
            root.display()
        );
    }
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let repository: Repository =
        serde_json::from_str(&raw).context("the manifest is not a valid Dialnode manifest")?;
    Ok(OpenRepository {
        repository,
        root: root.to_path_buf(),
    })
}

pub fn save_manifest(repository: &Repository, root: &Path) -> Result<()> {
    let path = root.join(MANIFEST_FILE_NAME);
    let mut text = serde_json::to_string_pretty(repository)?;
    text.push('\n');
    fs::write(&path, text).with_context(|| format!("failed to write {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_open() {
        let tmp = tempfile::tempdir().unwrap();
        let created = create("repo", tmp.path(), None).unwrap();
        assert!(created.root.join(MANIFEST_FILE_NAME).is_file());
        assert!(created.root.join(storage::DATABASE_FILE).is_file());

        let opened = open(&created.root).unwrap();
        assert_eq!(opened.repository, created.repository);

        assert!(create("repo", tmp.path(), None).is_err(), "already exists");
        assert!(create("a/b", tmp.path(), None).is_err());
        assert!(open(tmp.path()).is_err(), "no manifest");
    }

    #[test]
    fn old_sqlite_repositories_are_refused() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(MANIFEST_FILE_NAME), "{}").unwrap();
        fs::write(tmp.path().join(LEGACY_DATABASE), "").unwrap();
        let err = open(tmp.path()).err().unwrap().to_string();
        assert!(err.contains("old SQLite format"), "{err}");
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
        let repo = create("sample", &dir, None).unwrap();

        let mut entities = Entities::default();
        entities.speakers.push(Speaker::new("Jeet".into()));
        entities.skill_categories.push(SkillCategory {
            id: new_id(),
            name: "Body".into(),
            skills: vec![Skill::new("Strength".into())],
        });
        storage::save_entities(&repo.root, &entities).unwrap();

        let mut g = Graph::default();
        let start = g.add_node(NodeType::Start, pos2(0.0, 120.0));
        let ask = g.add_node(NodeType::Text, pos2(200.0, 80.0));
        let branches = g.add_node(NodeType::BranchContainer, pos2(560.0, 60.0));
        let blue = g.add_child(&branches, NodeType::Branch, None).unwrap();
        let lift = g
            .add_child(&branches, NodeType::SkillCheck, Some("Strength".into()))
            .unwrap();
        let reply = g.add_node(NodeType::Text, pos2(920.0, 60.0));
        let end = g.add_node(NodeType::End, pos2(1280.0, 140.0));
        if let NodeData::Text(d) = &mut g.node_mut(&ask).unwrap().data {
            d.text = "What's your favourite color?".into();
            d.speaker = Some("Jeet".into());
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
            d.speaker = Some("Jeet".into());
        }
        g.connect(&start, None, &ask, Some("left-target".into()));
        g.connect(&ask, Some("right-source".into()), &branches, None);
        g.connect(&blue, None, &reply, Some("left-target".into()));
        g.connect(&reply, Some("right-source".into()), &end, None);

        fs::create_dir(repo.root.join("Chapter 1")).unwrap();
        storage::write_dialogue(&repo.root.join("Chapter 1/Favourite colour.json"), &g).unwrap();
        storage::write_dialogue(&repo.root.join("Untitled.json"), &Graph::default()).unwrap();
        println!("{}", repo.root.display());
    }
}

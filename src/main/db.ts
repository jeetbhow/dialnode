import Database from "better-sqlite3";
import type {
  Portrait,
  Speaker,
  SkillCategory,
  Skill,
  SerializedDialogue,
  SerializedDialogueNode,
  SerializedDialogueEdge
} from "../shared/types";

const db = new Database("db.sqlite");

db.pragma("foreign_keys = ON");

db.exec(`
    CREATE TABLE IF NOT EXISTS dialogues (
      id TEXT PRIMARY KEY,
      name TEXT
    );
    CREATE TABLE IF NOT EXISTS nodes (
      id TEXT PRIMARY KEY,
      dialogueId TEXT,
      type TEXT,
      positionX REAL,
      positionY REAL,
      width REAL,
      height REAL,
      next TEXT,
      FOREIGN KEY (dialogueId) REFERENCES dialogues(id) ON DELETE CASCADE 
    );
    CREATE TABLE IF NOT EXISTS edges (
      id TEXT PRIMARY KEY,
      dialogueId TEXT,
      type TEXT,
      source TEXT,
      target TEXT,
      sourceHandle TEXT,
      targetHandle TEXT,
      FOREIGN KEY (dialogueId) REFERENCES dialogues(id) ON DELETE CASCADE
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
`);

export function closeDb(): void {
  db.close();
}

// Assumes better-sqlite3-style API: db.prepare(...), db.transaction(...), db.pragma(...)

export function saveDialogues(dialogues: SerializedDialogue[]): void {
  const deleteAllDialogues = db.prepare(`DELETE FROM dialogues`);

  const insertDialogue = db.prepare(`
    INSERT INTO dialogues (id, name)
    VALUES (?, ?)
  `);

  const insertNode = db.prepare(`
    INSERT INTO nodes (
      id, dialogueId, "type", positionX, positionY, width, height, "next"
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const insertEdge = db.prepare(`
    INSERT INTO edges (
      id, dialogueId, "type", source, target, sourceHandle, targetHandle
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
  `);

  const writeAll = db.transaction((all: SerializedDialogue[]) => {
    deleteAllDialogues.run(); // Since we have cascading deletes, we only have to delete the dialogues.

    for (const dialogue of all) {
      const dId = dialogue.id;
      insertDialogue.run(dId, dialogue.name);

      for (const n of dialogue.nodes) {
        insertNode.run(
          n.id,
          dId,
          n.type,
          n.positionX,
          n.positionY,
          n.width,
          n.height,
          n.next ?? null
        );
      }

      for (const e of dialogue.edges) {
        insertEdge.run(
          e.id,
          dId,
          e.type,
          e.source,
          e.target,
          e.sourceHandle ?? null,
          e.targetHandle ?? null
        );
      }
    }
  });

  try {
    writeAll(dialogues);
  } catch (err) {
    const e = err as Error;
    e.message = `saveDialogues failed: ${e.message}`;
    throw e;
  }
}

export function getAllDialogues(): Array<{
  id: string;
  name: string;
  nodes: SerializedDialogueNode[];
  edges: SerializedDialogueEdge[];
}> {
  const dialogues = db.prepare(`SELECT id, name FROM dialogues`).all() as SerializedDialogue[];
  const getNodes = db.prepare(`SELECT * FROM nodes WHERE dialogueId = ?`);
  const getEdges = db.prepare(`SELECT * FROM edges WHERE dialogueId = ?`);

  return dialogues.map((dialogue) => ({
    ...dialogue,
    nodes: getNodes.all(dialogue.id) as SerializedDialogueNode[],
    edges: getEdges.all(dialogue.id) as SerializedDialogueEdge[]
  }));
}

// --- Portraits CRUD ---
export function createPortrait(portrait: Portrait): void {
  const stmt = db.prepare(`INSERT INTO portraits (
      id,
      name,
      kind,
      dataURL,
      width,
      height,
      path,
      relPath,
      virtualPath,
      filename) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`);
  stmt.run(
    portrait.id,
    portrait.name,
    portrait.kind,
    portrait.dataURL,
    portrait.width,
    portrait.height,
    portrait.path,
    portrait.relPath,
    portrait.virtualPath,
    portrait.filename
  );
}

export function getAllPortraits(): Portrait[] {
  return db.prepare(`SELECT * FROM portraits`).all() as Portrait[];
}

export function deletePortrait(id: string): void {
  db.prepare(`DELETE FROM portraits WHERE id = ?`).run(id);
}

// --- Speakers CRUD ---
export function createSpeaker(speaker: Speaker): void {
  db.prepare(`INSERT INTO speakers (id, kind, name) VALUES (?, ?, ?)`).run(
    speaker.id,
    speaker.kind,
    speaker.name
  );
}

export function getAllSpeakers(): Speaker[] {
  return db.prepare(`SELECT * FROM speakers`).all() as Speaker[];
}

export function deleteSpeaker(id: string): void {
  db.prepare(`DELETE FROM speakers WHERE id = ?`).run(id);
}

// --- Skill Categories CRUD ---
export function createSkillCategory(category: SkillCategory): void {
  db.prepare(`INSERT INTO skillCategories (id, kind, name) VALUES (?, ?, ?)`).run(
    category.id,
    category.kind,
    category.name
  );
}

export function getAllSkillCategories(): SkillCategory[] {
  return db.prepare(`SELECT * FROM skillCategories`).all() as SkillCategory[];
}

export function deleteSkillCategory(id: string): void {
  db.prepare(`DELETE FROM skillCategories WHERE id = ?`).run(id);
}

// --- Skills CRUD ---
export function createSkill(skill: Skill): void {
  db.prepare(`INSERT INTO skills (id, kind, categoryId, name) VALUES (?, ?, ?, ?)`).run(
    skill.id,
    skill.kind,
    skill.categoryId,
    skill.name
  );
}

export function getAllSkills(): Skill[] {
  return db.prepare(`SELECT * FROM skills`).all() as Skill[];
}

export function deleteSkill(id: string): void {
  db.prepare(`DELETE FROM skills WHERE id = ?`).run(id);
}

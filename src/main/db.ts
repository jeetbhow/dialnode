import sqlite from "better-sqlite3";
import { join } from "path";

import type {
  Portrait,
  Speaker,
  SkillCategory,
  Skill,
  SerializedDialogue,
  SerializedGraphNode,
  SerializedGraphEdge,
  SerializedDialogueNode,
  SerializedFolder
} from "../shared/types";

const ERROR_MSG = "Database does not exist.";

let db: sqlite.Database | null = null;

export function dbExists(): boolean {
  return db !== null;
}

export function createDb(path: string): void {
  db = new sqlite(join(path, "db.sqlite"));
  db.pragma("foreign_keys = ON");
  db.exec(`
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
`);
}

export function closeDb(): void {
  if (db !== null) db.close();
}

export function connectDb(path: string): void {
  db = new sqlite(path);
  db.pragma("foreign_keys = ON");
}

export function saveDialogues(dialogues: SerializedDialogueNode[]): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  const deleteAllDialogueNodes = db.prepare(`DELETE FROM dialogueNodes`);

  const insertDialogueNode = db.prepare(`
    INSERT INTO dialogueNodes (id, type, name, parentId)
    VALUES (?, ?, ?, ?)
  `);

  const insertNode = db.prepare(`
    INSERT INTO nodes (
      id, parentId, extent, dialogueId, "type", positionX, positionY, width, height, "data"
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const insertEdge = db.prepare(`
    INSERT INTO edges (
      id, dialogueId, "type", source, target, sourceHandle, targetHandle
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
  `);

  const writeAll = db.transaction((all: SerializedDialogueNode[]) => {
    deleteAllDialogueNodes.run();

    for (const dialogue of all) {
      insertDialogueNode.run(dialogue.id, dialogue.type, dialogue.name, dialogue.parentId);
      if (dialogue.type === "dialogue") {
        for (const n of dialogue.nodes) {
          insertNode.run(
            n.id,
            n.parentId ?? null,
            n.extent ?? null,
            dialogue.id,
            n.type,
            n.positionX,
            n.positionY,
            n.width ?? null,
            n.height ?? null,
            n.data ?? null
          );
        }

        for (const e of dialogue.edges) {
          insertEdge.run(
            e.id,
            dialogue.id,
            e.type,
            e.source,
            e.target,
            e.sourceHandle ?? null,
            e.targetHandle ?? null
          );
        }
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

export function getAllDialogues(): SerializedDialogueNode[] {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  const dialogueNodes = db.prepare(`SELECT * FROM dialogueNodes`).all() as {
    id: string;
    type: "folder" | "dialogue";
    name: string;
    parentId: string | null;
  }[];

  const allGraphNodes = db.prepare(`SELECT * FROM nodes`).all() as SerializedGraphNode[];
  const nodesByDialogueId = allGraphNodes.reduce(
    (acc, node) => {
      if (!acc[node.dialogueId]) {
        acc[node.dialogueId] = [];
      }
      acc[node.dialogueId].push(node);
      return acc;
    },
    {} as Record<string, SerializedGraphNode[]>
  );

  const allGraphEdges = db.prepare(`SELECT * FROM edges`).all() as SerializedGraphEdge[];
  const edgesByDialogueId = allGraphEdges.reduce(
    (acc, edge) => {
      if (!acc[edge.dialogueId]) {
        acc[edge.dialogueId] = [];
      }
      acc[edge.dialogueId].push(edge);
      return acc;
    },
    {} as Record<string, SerializedGraphEdge[]>
  );

  return dialogueNodes.map((node) => {
    if (node.type === "dialogue") {
      return {
        id: node.id,
        type: "dialogue",
        name: node.name,
        parentId: node.parentId,
        nodes: nodesByDialogueId[node.id] || [],
        edges: edgesByDialogueId[node.id] || []
      } as SerializedDialogue;
    } else {
      return {
        id: node.id,
        type: "folder",
        name: node.name,
        parentId: node.parentId
      } as SerializedFolder;
    }
  });
}

// --- Portraits CRUD ---
export function createPortrait(portrait: Portrait): void {
  if (db === null) {
    return;
  }

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
  if (db === null) {
    throw new Error(ERROR_MSG);
  }
  return db.prepare(`SELECT * FROM portraits`).all() as Portrait[];
}

export function deletePortrait(id: string): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`DELETE FROM portraits WHERE id = ?`).run(id);
}

// --- Speakers CRUD ---
export function createSpeaker(speaker: Speaker): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`INSERT INTO speakers (id, kind, name) VALUES (?, ?, ?)`).run(
    speaker.id,
    speaker.kind,
    speaker.name
  );
}

export function getAllSpeakers(): Speaker[] {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  return db.prepare(`SELECT * FROM speakers`).all() as Speaker[];
}

export function deleteSpeaker(id: string): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`DELETE FROM speakers WHERE id = ?`).run(id);
}

// --- Skill Categories CRUD ---
export function createSkillCategory(category: SkillCategory): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`INSERT INTO skillCategories (id, kind, name) VALUES (?, ?, ?)`).run(
    category.id,
    category.kind,
    category.name
  );
}

export function getAllSkillCategories(): SkillCategory[] {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  return db.prepare(`SELECT * FROM skillCategories`).all() as SkillCategory[];
}

export function deleteSkillCategory(id: string): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`DELETE FROM skillCategories WHERE id = ?`).run(id);
}

// --- Skills CRUD ---
export function createSkill(skill: Skill): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`INSERT INTO skills (id, kind, categoryId, name) VALUES (?, ?, ?, ?)`).run(
    skill.id,
    skill.kind,
    skill.categoryId,
    skill.name
  );
}

export function getAllSkills(): Skill[] {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  return db.prepare(`SELECT * FROM skills`).all() as Skill[];
}

export function deleteSkill(id: string): void {
  if (db === null) {
    throw new Error(ERROR_MSG);
  }

  db.prepare(`DELETE FROM skills WHERE id = ?`).run(id);
}

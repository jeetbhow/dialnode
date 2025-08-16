import Database from "better-sqlite3";
import type { Portrait, Speaker, SkillCategory, Skill } from "../shared/types";

const db = new Database("db.sqlite");

db.exec(`
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

// --- Portraits CRUD ---
export function createPortrait(portrait: Portrait): void {
  const stmt = db.prepare(
    `INSERT INTO portraits (id, name, kind, dataURL, width, height, path, relPath, virtualPath, filename) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
  );
  stmt.run(
    portrait.id,
    portrait.kind,
    portrait.name,
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

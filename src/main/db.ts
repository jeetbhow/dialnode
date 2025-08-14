import Database from "better-sqlite3";
import type { Portrait, Speaker, SkillCategory, Skill } from "../shared/types";

const db = new Database("db.dqlite");

db.exec(`
    CREATE TABLE IF NOT EXISTS portraits (
        id TEXT PRIMARY KEY,
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
        name TEXT
    );
    CREATE TABLE IF NOT EXISTS skill_categories (
        id TEXT PRIMARY KEY,
        name TEXT
    );
    CREATE TABLE IF NOT EXISTS skills (
        id TEXT PRIMARY KEY,
        category_id TEXT,
        name TEXT,
        FOREIGN KEY (category_id) REFERENCES skill_categories(id)
    );
`);

export function closeDb() {
  db.close();
}

// --- Portraits CRUD ---
export function createPortrait(portrait: Portrait): void {
  const stmt = db.prepare(
    `INSERT INTO portraits (id, name, dataURL, width, height, path, relPath, virtualPath, filename) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)`
  );
  stmt.run(
    portrait.id,
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

export function getPortrait(id: string): Portrait | undefined {
  return db.prepare(`SELECT * FROM portraits WHERE id = ?`).get(id) as Portrait | undefined;
}

export function getAllPortraits(): Portrait[] {
  return db.prepare(`SELECT * FROM portraits`).all() as Portrait[];
}

export function updatePortrait(portrait: Portrait): void {
  const stmt = db.prepare(
    `UPDATE portraits SET name = ?, dataURL = ?, width = ?, height = ?, path = ?, relPath = ?, virtualPath = ?, filename = ? WHERE id = ?`
  );
  stmt.run(
    portrait.name,
    portrait.dataURL,
    portrait.width,
    portrait.height,
    portrait.path,
    portrait.relPath,
    portrait.virtualPath,
    portrait.filename,
    portrait.id
  );
}

export function deletePortrait(id: string): void {
  db.prepare(`DELETE FROM portraits WHERE id = ?`).run(id);
}

// --- Speakers CRUD ---
export function createSpeaker(speaker: Speaker): void {
  db.prepare(`INSERT INTO speakers (id, name) VALUES (?, ?)`).run(speaker.id, speaker.name);
}

export function getSpeaker(id: string): Speaker | undefined {
  return db.prepare(`SELECT * FROM speakers WHERE id = ?`).get(id) as Speaker | undefined;
}

export function getAllSpeakers(): Speaker[] {
  return db.prepare(`SELECT * FROM speakers`).all() as Speaker[];
}

export function updateSpeaker(speaker: Speaker): void {
  db.prepare(`UPDATE speakers SET name = ? WHERE id = ?`).run(speaker.name, speaker.id);
}

export function deleteSpeaker(id: string): void {
  db.prepare(`DELETE FROM speakers WHERE id = ?`).run(id);
}

// --- Skill Categories CRUD ---
export function createSkillCategory(category: SkillCategory): void {
  db.prepare(`INSERT INTO skill_categories (id, name) VALUES (?, ?)`).run(
    category.id,
    category.name
  );
}

export function getSkillCategory(id: string): SkillCategory | undefined {
  return db.prepare(`SELECT * FROM skill_categories WHERE id = ?`).get(id) as
    | SkillCategory
    | undefined;
}

export function getAllSkillCategories(): SkillCategory[] {
  return db.prepare(`SELECT * FROM skill_categories`).all() as SkillCategory[];
}

export function updateSkillCategory(category: SkillCategory): void {
  db.prepare(`UPDATE skill_categories SET name = ? WHERE id = ?`).run(category.name, category.id);
}

export function deleteSkillCategory(id: string): void {
  db.prepare(`DELETE FROM skill_categories WHERE id = ?`).run(id);
}

// --- Skills CRUD ---
export function createSkill(skill: Skill): void {
  db.prepare(`INSERT INTO skills (id, category_id, name) VALUES (?, ?, ?)`).run(
    skill.id,
    skill.category.id,
    skill.name
  );
}

export function getSkill(id: string): Skill | undefined {
  return db.prepare(`SELECT * FROM skills WHERE id = ?`).get(id) as Skill | undefined;
}

export function getAllSkills(): Skill[] {
  return db.prepare(`SELECT * FROM skills`).all() as Skill[];
}

export function updateSkill(skill: Skill): void {
  db.prepare(`UPDATE skills SET category_id = ?, name = ? WHERE id = ?`).run(
    skill.category.id,
    skill.name,
    skill.id
  );
}

export function deleteSkill(id: string): void {
  db.prepare(`DELETE FROM skills WHERE id = ?`).run(id);
}

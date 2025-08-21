import type { Portrait, Speaker, SkillCategory, DbEntity } from "../../../../shared/types";

type Database = {
  portraits: Portrait[];
  speakers: Speaker[];
  skillCategories: SkillCategory[];
  selectedId: string;
};

const db = $state<Database>({
  portraits: [] as Portrait[],
  speakers: [] as Speaker[],
  skillCategories: [] as SkillCategory[],
  selectedId: ""
});

export async function loadSpeakersFromDb(): Promise<void> {
  db.speakers = await window.api.getAllSpeakers();
}

export async function loadPortraitsFromDb(): Promise<void> {
  db.portraits = await window.api.getAllPortraits();
}

export async function loadSkillsFromDb(): Promise<void> {
  const categories = await window.api.getAllSkillCategories();
  db.skillCategories = categories.map((category) => ({ ...category, skills: [] }));

  const fetchedSkills = await window.api.getAllSkills();
  for (const category of db.skillCategories) {
    category.skills = fetchedSkills.filter((skill) => skill.categoryId === category.id);
  }
}

export async function addEntity(entity: DbEntity): Promise<void> {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits.push(entity);
      await window.api.createPortrait(entity);
      break;
    case "skillCategory":
      db.skillCategories.push(entity);
      await window.api.createSkillCategory(entity);
      break;
    case "skill": {
      const category = db.skillCategories.find((category) => (category.id = entity.categoryId));
      category.skills.push(entity);
      await window.api.createSkill(entity);
      break;
    }
    case "speaker":
      db.speakers.push(entity);
      await window.api.createSpeaker(entity);
      break;
  }
}

export function selectEntity(entity: DbEntity): void {
  db.selectedId = entity.id;
}

export async function deleteEntity(entity: DbEntity): Promise<void> {
  if (!entity) {
    return;
  }

  switch (entity.kind) {
    case "portrait":
      db.portraits = db.portraits.filter((p) => p.id !== entity.id);
      await window.api.deletePortrait(entity.id);
      break;
    case "skillCategory":
      db.skillCategories = db.skillCategories.filter((s) => s.id !== entity.id);
      await window.api.deleteSkillCategory(entity.id);
      break;
    case "skill": {
      const category = db.skillCategories.find((category) => category.id === entity.categoryId);
      category.skills = category.skills.filter((skill) => skill.id !== entity.id);
      await window.api.deleteSkill(entity.id);
      break;
    }
    case "speaker":
      db.speakers = db.speakers.filter((s) => s.id !== entity.id);
      await window.api.deleteSpeaker(entity.id);
      break;
  }

  if (entity.id === db.selectedId) {
    db.selectedId = "";
  }
}

export function useDb(): Database {
  return db;
}

import type { Portrait, Speaker, SkillCategory, DbEntity } from "../../../shared/types";

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
  const speakers = await window.api.getAllSpeakers();
  db.speakers = (speakers ?? []).map((s) => ({ ...s, kind: "speaker" }));
}

export async function loadPortraitsFromDb(): Promise<void> {
  const portraits = await window.api.getAllPortraits();
  db.portraits = (portraits ?? []).map((p) => ({ ...p, kind: "portrait" }));
}

export async function loadSkillsFromDb(): Promise<void> {
  const categories = await window.api.getAllSkillCategories();
  db.skillCategories = (categories ?? []).map((c) => ({
    ...c,
    kind: "skill-category",
    skills: []
  }));

  const skills = await window.api.getAllSkills();
  (skills ?? []).forEach((skill) => {
    skill.kind = "skill";
    const category = db.skillCategories.find((c) => c.id === skill.category_id);
    if (category) {
      category.skills.push(skill);
    }
  });
}

export function addEntity(entity: DbEntity): void {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits = [...db.portraits, entity];
      window.api.createPortrait(entity);
      break;
    case "skill-category":
      db.skillCategories = [...db.skillCategories, entity];
      window.api.createSkillCategory(entity);
      break;
    case "skill": {
      entity.category.skills.push(entity);
      const skillPayload = {
        id: entity.id,
        category_id: entity.category.id,
        name: entity.name,
        kind: "skill"
      };
      window.api.createSkill(skillPayload);
      break;
    }
    case "speaker":
      db.speakers = [...db.speakers, entity];
      window.api.createSpeaker(entity);
      break;
  }
}

export function selectEntity(entity: DbEntity): void {
  db.selectedId = entity.id;
}

export function deleteEntity(entity: DbEntity): void {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits = db.portraits.filter((p) => p.id !== entity.id);
      window.api.deletePortrait(entity.id);
      break;
    case "skill-category":
      db.skillCategories = db.skillCategories.filter((s) => s.id !== entity.id);
      window.api.deleteSkillCategory(entity.id);
      break;
    case "skill":
      entity.category.skills = entity.category.skills.filter((s) => s.id !== entity.id);
      window.api.deleteSkill(entity.id);
      break;
    case "speaker":
      db.speakers = db.speakers.filter((s) => s.id !== entity.id);
      window.api.deleteSpeaker(entity.id);
      break;
  }
}

export function useDb(): Database {
  return db;
}

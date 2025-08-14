import type { Portrait, Speaker, SkillCategory, DbEntity } from "../../../shared/types";

const db = $state({
  portraits: [] as Portrait[],
  speakers: [] as Speaker[],
  skillCategories: [] as SkillCategory[],
  selectedId: ""
});

// Load speakers from the database on app start
export async function loadSpeakersFromDb() {
  const speakers = await window.api.getAllSpeakers();
  db.speakers = (speakers ?? []).map((s) => ({ ...s, kind: "speaker" }));
}

export function addEntity(entity: DbEntity) {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits = [...db.portraits, entity];
      break;
    case "skill-category":
      db.skillCategories = [...db.skillCategories, entity];
      break;
    case "skill":
      entity.category.skills.push(entity);
      break;
    case "speaker":
      db.speakers = [...db.speakers, entity];
      window.api.createSpeaker(entity);
      break;
  }
}

export function selectEntity(entity: DbEntity) {
  db.selectedId = entity.id;
}

export async function deleteEntity(entity: DbEntity) {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits = db.portraits.filter((p) => p.id !== entity.id);
      break;
    case "skill-category":
      db.skillCategories = db.skillCategories.filter((s) => s.id !== entity.id);
      break;
    case "skill":
      entity.category.skills = entity.category.skills.filter((s) => s.id !== entity.id);
      break;
    case "speaker":
      db.speakers = db.speakers.filter((s) => s.id !== entity.id);
      window.api.deleteSpeaker(entity.id);
      break;
  }
}

export function useDb() {
  return db;
}

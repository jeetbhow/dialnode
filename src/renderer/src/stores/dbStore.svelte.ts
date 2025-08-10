export type Portrait = {
  id: string;
  kind: "portrait";
  name: string;
  dataURL: string;
  width: number;
  height: number;
  path: string;
  relPath: string;
  virtualPath: string;
  filename: string;
};

export type Speaker = {
  id: string;
  kind: "speaker";
  name: string;
};

export type SkillType = "body" | "mind" | "psyche" | "sense";

export type SkillCategory = {
  id: string;
  kind: "skill-category";
  name: string;
  skills: Skill[];
};

export type Skill = {
  id: string;
  kind: "skill";
  category: SkillCategory;
  name: string;
};

export type DbEntityKind = "portrait" | "speaker" | "skill-category" | "skill";
export type DbEntity = Speaker | Portrait | SkillCategory | Skill;

const db = $state({
  portraits: [] as Portrait[],
  speakers: [] as Speaker[],
  skillCategories: [] as SkillCategory[],
  selectedId: ""
});

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
      break;
  }
}

export function selectEntity(entity: DbEntity) {
  db.selectedId = entity.id;
}

export function deleteEntity(entity: DbEntity) {
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
      break;
  }
}

export function useDb() {
  return db;
}

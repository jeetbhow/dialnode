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

export type SkillType = "Body" | "Mind" | "Psyche" | "Sense";
export type Skill = {
  id: string;
  kind: "skill";
  name: string;
  color: string;
  type: string;
};

export type DbEntityKind = "portrait" | "speaker" | "skill";
export type DbEntity = Speaker | Portrait | Skill;

const db = $state({
  portraits: [] as Portrait[],
  speakers: [] as Speaker[],
  skills: [] as Skill[],
  selectedId: ""
});

export function addEntity(entity: DbEntity) {
  if (!entity) return;

  switch (entity.kind) {
    case "portrait":
      db.portraits = [...db.portraits, entity];
      break;
    case "skill":
      db.skills = [...db.skills, entity];
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
    case "skill":
      db.skills = db.skills.filter((s) => s.id !== entity.id);
      break;
    case "speaker":
      db.speakers = db.speakers.filter((s) => s.id !== entity.id);
      break;
  }
}

export function useDb() {
  return db;
}

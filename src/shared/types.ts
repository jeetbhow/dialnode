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

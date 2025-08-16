import { ElectronAPI } from "@electron-toolkit/preload";
import type { Skill, SkillCategory, Speaker, Portrait } from "../shared/types";

type ImageMetaData = {
  dataURL: string;
  width: number;
  height: number;
  type?: string;
  path: string;
  relPath: string;
  filename: string;
};

interface AppApi {
  getAllSkills: () => Promise<Skill[]>;
  createSkill: (skill: Skill) => Promise<void>;
  deleteSkill: (skillId: string) => Promise<void>;
  getAllSkillCategories: () => Promise<SkillCategory[]>;
  createSkillCategory: (category: SkillCategory) => Promise<void>;
  deleteSkillCategory: (categoryId: string) => Promise<void>;
  selectImage: (projectDir: string) => Promise<ImageMetaData | null>;
  selectDirectory: () => Promise<string | null>;
  exportJson: (data: Record<string, unknown>[]) => Promise<boolean>;
  createSpeaker: (speaker: Speaker) => Promise<void>;
  deleteSpeaker: (speakerId: string) => Promise<void>;
  getAllSpeakers: () => Promise<Speaker[]>;
  getAllPortraits: () => Promise<Portrait[]>;
  createPortrait: (portrait: Portrait) => Promise<void>;
  deletePortrait: (portraitId: string) => Promise<void>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

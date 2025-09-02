import { ElectronAPI } from "@electron-toolkit/preload";
import type {
  Skill,
  SkillCategory,
  Speaker,
  Portrait,
  SerializedDialogue,
  ElectronSelectDirectoryOptions,
  ExtraSelectDirectoryOptions,
  Repository,
  DialogueJSON,
  SerializedDialogueNode
} from "../shared/types";

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
  minimize: () => void;
  maximize: () => void;
  close: () => void;
  createRepositoryWindow: () => void;

  waitForRepo: () => Promise<Repository>;

  saveDialogues: (dialogues: SerializedDialogueNode[]) => Promise<void>;
  getAllDialogues: () => Promise<SerializedDialogueNode[]>;

  getAllSkills: () => Promise<Skill[]>;
  createSkill: (skill: Skill) => Promise<void>;
  deleteSkill: (skillId: string) => Promise<void>;
  getAllSkillCategories: () => Promise<SkillCategory[]>;
  createSkillCategory: (category: SkillCategory) => Promise<void>;
  deleteSkillCategory: (categoryId: string) => Promise<void>;
  createSpeaker: (speaker: Speaker) => Promise<void>;
  deleteSpeaker: (speakerId: string) => Promise<void>;
  getAllSpeakers: () => Promise<Speaker[]>;
  getAllPortraits: () => Promise<Portrait[]>;
  createPortrait: (portrait: Portrait) => Promise<void>;
  deletePortrait: (portraitId: string) => Promise<void>;

  selectImage: (projectDir: string) => Promise<ImageMetaData | null>;
  selectDirectory: (
    options: ElectronSelectDirectoryOptions,
    extraOptions?: ExtraSelectDirectoryOptions
  ) => Promise<string | null>;
  createRepository: (repository: Repository) => Promise<void>;
  openRepository: () => Promise<void>;
  exportJson: (data: DialogueJSON[]) => Promise<void>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

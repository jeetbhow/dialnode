import { ElectronAPI } from "@electron-toolkit/preload";

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
  getAllSkillCategories: () => Promise<SkillCategory[]>;
  createSkillCategory: (category: { id: string; name: string; kind: string }) => Promise<void>;
  deleteSkillCategory: (categoryId: string) => Promise<void>;
  selectImage: (projectDir: string) => Promise<ImageMetaData | null>;
  selectDirectory: () => Promise<string | null>;
  exportJson: (data: Record<string, unknown>[]) => Promise<boolean>;
  createSpeaker: (speaker: { id: string; name: string; kind: string }) => Promise<void>;
  deleteSpeaker: (speakerId: string) => Promise<void>;
  getAllSpeakers: () => Promise<Speaker[]>;
  getAllPortraits: () => Promise<Portrait[]>;
  createPortrait: (portrait: {
    id: string;
    name: string;
    dataURL: string;
    width: number;
    height: number;
    path: string;
    relPath: string;
    virtualPath: string;
    filename: string;
    kind: string;
  }) => Promise<void>;
  deletePortrait: (portraitId: string) => Promise<void>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

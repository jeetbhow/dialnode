import { contextBridge, ipcRenderer } from "electron";
import { electronAPI } from "@electron-toolkit/preload";
import {
  Portrait,
  SerializedDialogue,
  Skill,
  SkillCategory,
  Speaker,
  type DialogueJSON,
  type ElectronSelectDirectoryOptions,
  type ExtraSelectDirectoryOptions,
  type Repository
} from "../shared/types";

let latestRepo: Repository | null = null;

ipcRenderer.on("repository-opened", (_evt, repo: Repository) => {
  latestRepo = repo;
});

// Custom APIs for renderer
const api = {
  // Window
  minimize: () => ipcRenderer.send("window-minimize"),
  maximize: () => ipcRenderer.send("window-maximize"),
  close: () => ipcRenderer.send("window-close"),
  createRepositoryWindow: (): void => {
    ipcRenderer.send("open-repository-window");
  },

  waitForRepo: (): Promise<Repository> => {
    if (latestRepo) {
      return Promise.resolve(latestRepo);
    } else {
      return new Promise((resolve) =>
        ipcRenderer.once("repository-opened", (_e, repo: Repository) => resolve(repo))
      );
    }
  },

  // DB
  getAllDialogues: async () => await ipcRenderer.invoke("get-all-dialogues"),
  saveDialogues: (dialogues: SerializedDialogue[]) =>
    ipcRenderer.invoke("save-dialogues", dialogues),
  getAllSkills: async () => await ipcRenderer.invoke("get-all-skills"),
  createSkill: async (skill: Skill) => await ipcRenderer.invoke("create-skill", skill),
  deleteSkill: async (skillId: string) => await ipcRenderer.invoke("delete-skill", skillId),
  getAllSkillCategories: async () => await ipcRenderer.invoke("get-all-skill-categories"),
  createSkillCategory: async (category: SkillCategory) =>
    await ipcRenderer.invoke("create-skill-category", category),
  deleteSkillCategory: async (categoryId: string) =>
    await ipcRenderer.invoke("delete-skill-category", categoryId),
  getAllSpeakers: async () => await ipcRenderer.invoke("get-all-speakers"),
  createSpeaker: async (speaker: Speaker) => await ipcRenderer.invoke("create-speaker", speaker),
  deleteSpeaker: async (speakerId: string) => await ipcRenderer.invoke("delete-speaker", speakerId),
  getAllPortraits: async () => await ipcRenderer.invoke("get-all-portraits"),
  createPortrait: async (portrait: Portrait) =>
    await ipcRenderer.invoke("create-portrait", portrait),
  deletePortrait: async (portraitId: string) =>
    await ipcRenderer.invoke("delete-portrait", portraitId),

  // File
  selectImage: async (projectDir: string) => await ipcRenderer.invoke("select-image", projectDir),
  selectDirectory: async (
    options: ElectronSelectDirectoryOptions,
    extraOptions?: ExtraSelectDirectoryOptions
  ) => await ipcRenderer.invoke("select-directory", options, extraOptions),
  createRepository: async (repository: Repository) =>
    await ipcRenderer.invoke("create-repository", repository),
  openRepository: async () => await ipcRenderer.invoke("open-repository"),
  exportJson: async (data: DialogueJSON) => await ipcRenderer.invoke("export-json", data)
};

// Use `contextBridge` APIs to expose Electron APIs to
// renderer only if context isolation is enabled, otherwise
// just add to the DOM global.
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld("electron", electronAPI);
    contextBridge.exposeInMainWorld("api", api);
  } catch (error) {
    console.error(error);
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI;
  // @ts-ignore (define in dts)
  window.api = api;
}

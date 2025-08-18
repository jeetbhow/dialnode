import { contextBridge, ipcRenderer } from "electron";
import { electronAPI } from "@electron-toolkit/preload";
import { Portrait, SerializedDialogue, Skill, SkillCategory, Speaker } from "../shared/types";

// Custom APIs for renderer
const api = {
  minimize: () => ipcRenderer.send("window-minimize"),
  maximize: () => ipcRenderer.send("window-maximize"),
  close: () => ipcRenderer.send("window-close"),
  getAllDialogues: async () => {
    return await ipcRenderer.invoke("get-all-dialogues");
  },
  saveDialogues: (dialogues: SerializedDialogue[]) => {
    ipcRenderer.invoke("save-dialogues", dialogues);
  },
  getAllSkills: async () => {
    return await ipcRenderer.invoke("get-all-skills");
  },
  createSkill: async (skill: Skill) => {
    return await ipcRenderer.invoke("create-skill", skill);
  },
  deleteSkill: async (skillId: string) => {
    return await ipcRenderer.invoke("delete-skill", skillId);
  },
  getAllSkillCategories: async () => {
    return await ipcRenderer.invoke("get-all-skill-categories");
  },
  createSkillCategory: async (category: SkillCategory) => {
    return await ipcRenderer.invoke("create-skill-category", category);
  },
  deleteSkillCategory: async (categoryId: string) => {
    return await ipcRenderer.invoke("delete-skill-category", categoryId);
  },
  selectImage: async (projectDir: string) => {
    return await ipcRenderer.invoke("select-image", projectDir);
  },
  selectDirectory: async () => {
    return await ipcRenderer.invoke("select-directory");
  },
  exportJson: async (data: Record<string, unknown>[]) => {
    return await ipcRenderer.invoke("export-json", data);
  },
  getAllSpeakers: async () => {
    return await ipcRenderer.invoke("get-all-speakers");
  },
  createSpeaker: async (speaker: Speaker) => {
    return await ipcRenderer.invoke("create-speaker", speaker);
  },
  deleteSpeaker: async (speakerId: string) => {
    return await ipcRenderer.invoke("delete-speaker", speakerId);
  },
  getAllPortraits: async () => {
    return await ipcRenderer.invoke("get-all-portraits");
  },
  createPortrait: async (portrait: Portrait) => {
    return await ipcRenderer.invoke("create-portrait", portrait);
  },
  deletePortrait: async (portraitId: string) => {
    return await ipcRenderer.invoke("delete-portrait", portraitId);
  }
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

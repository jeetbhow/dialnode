import { contextBridge, ipcRenderer } from "electron";
import { electronAPI } from "@electron-toolkit/preload";

// Custom APIs for renderer
const api = {
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
  createSpeaker: async (speaker: { id: string; name: string; kind: string }) => {
    return await ipcRenderer.invoke("create-speaker", speaker);
  },
  deleteSpeaker: async (speakerId: string) => {
    return await ipcRenderer.invoke("delete-speaker", speakerId);
  },
  getAllPortraits: async () => {
    return await ipcRenderer.invoke("get-all-portraits");
  },
  createPortrait: async (portrait: {
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
  }) => {
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

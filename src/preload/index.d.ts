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
  selectImage: (projectDir: string) => Promise<ImageMetaData | null>;
  selectDirectory: () => Promise<string | null>;
  exportJson: (data: Record<string, unknown>[]) => Promise<boolean>;
  createSpeaker: (speaker: { id: string; name: string; kind: string }) => Promise<void>;
  deleteSpeaker: (speakerId: string) => Promise<void>;
  getAllSpeakers: () => Promise<Speaker[]>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

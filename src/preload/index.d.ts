import { ElectronAPI } from '@electron-toolkit/preload';

type ImageMetaData = {
  dataURL: string;
  width: number;
  height: number;
  type?: string;
  path: string;
  filename: string;
};

interface AppApi {
  selectImage: () => Promise<ImageMetaData | null>;
  selectDirectory: () => Promise<string | null>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

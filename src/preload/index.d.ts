import { ElectronAPI } from '@electron-toolkit/preload';

interface AppApi {
  selectImage: () => Promise<string | null>;
}

declare global {
  interface Window {
    electron: ElectronAPI;
    api: AppApi;
  }
}

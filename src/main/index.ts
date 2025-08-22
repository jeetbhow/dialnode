import {
  closeDb,
  saveDialogues,
  getAllSpeakers,
  createSpeaker,
  deleteSpeaker,
  getAllPortraits,
  createPortrait,
  deletePortrait,
  getAllSkillCategories,
  createSkillCategory,
  deleteSkillCategory,
  getAllSkills,
  createSkill,
  deleteSkill,
  getAllDialogues,
  createDb,
  dbExists,
  connectDb
} from "./db";

import { app, shell, BrowserWindow, ipcMain, dialog } from "electron";
import { electronApp, optimizer, is } from "@electron-toolkit/utils";
import icon from "../../resources/icon.png?asset";

import { join, basename, extname, relative } from "path";
import { readFileSync } from "fs";
import { stat, mkdir, writeFile, rm, readFile } from "fs/promises";

import { imageSize } from "image-size";
import type {
  DialogueJSON,
  SerializedDialogue,
  ElectronSelectDirectoryOptions,
  ExtraSelectDirectoryOptions,
  Repository
} from "../shared/types";

function getMimeType(filePath: string): string {
  const ext = extname(filePath).toLowerCase();
  switch (ext) {
    case ".jpg":
    case ".jpeg":
      return "image/jpeg";
    case ".png":
      return "image/png";
    case ".gif":
      return "image/gif";
    case ".bmp":
      return "image/bmp";
    case ".webp":
      return "image/webp";
    case ".svg":
      return "image/svg+xml";
    default:
      return "application/octet-stream";
  }
}

// Titlebar operations
ipcMain.on("window-minimize", () => {
  BrowserWindow.getFocusedWindow()?.minimize();
});

ipcMain.on("window-maximize", () => {
  const win = BrowserWindow.getFocusedWindow();
  if (win) {
    win.isMaximized() ? win.unmaximize() : win.maximize();
  }
});

ipcMain.on("window-close", () => {
  BrowserWindow.getFocusedWindow()?.close();
});

ipcMain.handle("save-dialogues", async (_event, dialogues: SerializedDialogue[]) => {
  return saveDialogues(dialogues);
});

ipcMain.handle("get-all-dialogues", async () => {
  return getAllDialogues();
});

// --- DB IPC Handlers ---
ipcMain.handle("get-all-speakers", async () => {
  return getAllSpeakers();
});

ipcMain.handle("create-speaker", async (_event, speaker) => {
  createSpeaker(speaker);
  return true;
});

ipcMain.handle("delete-speaker", async (_event, speakerId) => {
  deleteSpeaker(speakerId);
  return true;
});

ipcMain.handle("get-all-portraits", async () => {
  return getAllPortraits();
});

ipcMain.handle("create-portrait", async (_event, portrait) => {
  createPortrait(portrait);
  return true;
});

ipcMain.handle("delete-portrait", async (_event, portraitId) => {
  deletePortrait(portraitId);
  return true;
});

ipcMain.handle("get-all-skill-categories", async () => {
  return getAllSkillCategories();
});

ipcMain.handle("create-skill-category", async (_event, category) => {
  createSkillCategory(category);
  return true;
});

ipcMain.handle("delete-skill-category", async (_event, categoryId) => {
  deleteSkillCategory(categoryId);
  return true;
});

ipcMain.handle("get-all-skills", async () => {
  return getAllSkills();
});

ipcMain.handle("create-skill", async (_event, skill) => {
  createSkill(skill);
  return true;
});

ipcMain.handle("delete-skill", async (_event, skillId) => {
  deleteSkill(skillId);
  return true;
});

ipcMain.handle(
  "export-json",
  async (event, data: DialogueJSON[], defaultFileName: string = "data.json"): Promise<void> => {
    const win = BrowserWindow.fromWebContents(event.sender);

    if (!win) {
      throw Error("Failed to find window.");
    }

    const { canceled, filePath } = await dialog.showSaveDialog(win, {
      title: "Save JSON",
      defaultPath: defaultFileName,
      filters: [{ name: "JSON Files", extensions: ["json"] }],
      properties: ["showOverwriteConfirmation"]
    });

    if (canceled || !filePath) return Promise.reject();

    return writeFile(filePath, JSON.stringify(data, null, 2));
  }
);

ipcMain.handle("select-image", async (_event, projectDir: string) => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    properties: ["openFile"],
    filters: [{ name: "Images", extensions: ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"] }]
  });

  if (canceled || !filePaths.length) return null;

  const buf = readFileSync(filePaths[0]);
  const base64 = buf.toString("base64");
  const mimeType = getMimeType(filePaths[0]);
  const dataURL = `data:${mimeType};base64,${base64}`;
  const metadata = imageSize(buf);
  const path = filePaths[0];

  return {
    dataURL,
    width: metadata.width,
    height: metadata.height,
    type: metadata.type,
    path,
    relPath: relative(projectDir, path),
    filename: basename(path)
  };
});

ipcMain.handle("open-repository", async (): Promise<void> => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    title: "Open Repository",
    buttonLabel: "Select Folder",
    filters: [{ name: "Dialnode (.dial)", extensions: ["dial"] }],
    properties: ["openFile"]
  });

  if (canceled || !filePaths) {
    return;
  }

  // TODO Add error handling for this later.
  const manifestRaw = await readFile(filePaths[0], "utf-8");
  const repository: Repository = JSON.parse(manifestRaw);
  const dbPath = join(repository.location, repository.name, "db.sqlite");

  connectDb(dbPath);

  const windows = BrowserWindow.getAllWindows();
  for (const window of windows) {
    window.close();
  }
  createMainWindow(repository);
});

ipcMain.handle("create-repository", async (_event, repository: Repository) => {
  const repositoryPath = join(repository.location, repository.name);
  try {
    await mkdir(repositoryPath);
    createDb(repositoryPath);
    await writeFile(join(repositoryPath, "manifest.dial"), JSON.stringify(repository));

    const windows = BrowserWindow.getAllWindows();
    for (const window of windows) {
      window.close();
    }

    createMainWindow(repository);
  } catch (error) {
    const e = error as Error;
    await rm(repositoryPath, { force: true });
    throw new Error(`Failed to create the repository: ${e.message}`);
  }
});

ipcMain.handle(
  "select-directory",
  async (
    _event,
    options: ElectronSelectDirectoryOptions,
    extraOptions?: ExtraSelectDirectoryOptions
  ) => {
    const { canceled, filePaths } = await dialog.showOpenDialog({
      ...options,
      properties: ["openDirectory", ...(options.properties ?? [])]
    });

    if (canceled || !filePaths.length) return null;

    if (extraOptions?.godot === true) {
      try {
        const godotProjectFile = join(filePaths[0], "project.godot");
        const stats = await stat(godotProjectFile);
        if (!stats.isFile()) {
          throw new Error("Failed to find project.godot file.");
        }
      } catch (error) {
        const e = error as Error;
        throw new Error(`Failed to find project.godot file: ${e.message}`);
      }
    }

    return filePaths[0];
  }
);

ipcMain.on("open-repository-window", async () => {
  createRepositoryWindow();
});

function createMainWindow(repository: Repository): void {
  const mainWindow = new BrowserWindow({
    width: 1200,
    height: 900,
    show: false,
    frame: false,
    titleBarStyle: "hidden",
    autoHideMenuBar: true,
    ...(process.platform === "linux" ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, "../preload/index.js"),
      sandbox: false
    }
  });

  mainWindow.on("ready-to-show", () => {
    mainWindow.show();
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(`${process.env["ELECTRON_RENDERER_URL"]}?view=main`);
  } else {
    mainWindow.loadFile(join(__dirname, "../renderer/index.html"), { query: { view: "main" } });
  }

  mainWindow.webContents.on("did-finish-load", () => {
    mainWindow.webContents.send("repository-opened", repository);
  });
}

function createRepositoryWindow(): void {
  const repositoryWindow = new BrowserWindow({
    width: 750,
    height: 750,
    show: false,
    frame: false,
    titleBarStyle: "hidden",
    autoHideMenuBar: true,
    ...(process.platform === "linux" ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, "../preload/index.js"),
      sandbox: false
    }
  });

  repositoryWindow.on("ready-to-show", () => {
    repositoryWindow.show();
  });

  repositoryWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    repositoryWindow.loadURL(`${process.env["ELECTRON_RENDERER_URL"]}?view=project`);
  } else {
    repositoryWindow.loadFile(join(__dirname, "../renderer/index.html"), {
      query: { view: "project" }
    });
  }
}

app.whenReady().then(() => {
  electronApp.setAppUserModelId("com.electron");

  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  // IPC test
  ipcMain.on("ping", () => console.log("pong"));

  createRepositoryWindow();

  app.on("activate", function () {
    if (BrowserWindow.getAllWindows().length === 0) createRepositoryWindow();
  });
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("before-quit", () => {
  if (dbExists()) {
    closeDb();
  }
});

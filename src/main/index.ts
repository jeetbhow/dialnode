import { getAllSpeakers, createSpeaker, deleteSpeaker } from "./db";
import { closeDb } from "./db";

import { app, shell, BrowserWindow, ipcMain, dialog } from "electron";
import { electronApp, optimizer, is } from "@electron-toolkit/utils";
import icon from "../../resources/icon.png?asset";

import { join, basename, extname, relative } from "path";
import { readFileSync, writeFileSync } from "fs";

import { imageSize } from "image-size";

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

// Handle save request from renderer
ipcMain.handle("export-json", async (event, data, defaultFileName = "data.json") => {
  const win = BrowserWindow.fromWebContents(event.sender);

  if (!win) {
    return false;
  }

  const { canceled, filePath } = await dialog.showSaveDialog(win, {
    title: "Save JSON",
    defaultPath: defaultFileName,
    filters: [{ name: "JSON Files", extensions: ["json"] }],
    properties: ["showOverwriteConfirmation"]
  });

  if (canceled || !filePath) return false;

  const jsonString = JSON.stringify(data, null, 2);
  writeFileSync(filePath, jsonString);

  return true;
});

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

ipcMain.handle("select-directory", async () => {
  const { canceled, filePaths } = await dialog.showOpenDialog({
    properties: ["openDirectory"]
  });

  if (canceled || !filePaths.length) return null;
  return filePaths[0];
});

function createWindow(): void {
  // Create the browser window
  const mainWindow = new BrowserWindow({
    width: 1280,
    height: 720,
    show: false,
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

  // HMR for renderer base on electron-vite cli.
  // Load the remote URL for development or the local html file for production.
  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(process.env["ELECTRON_RENDERER_URL"]);
  } else {
    mainWindow.loadFile(join(__dirname, "../renderer/index.html"));
  }
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  // Set app user model id for windows
  electronApp.setAppUserModelId("com.electron");

  // Default open or close DevTools by F12 in development
  // and ignore CommandOrControl + R in production.
  // see https://github.com/alex8088/electron-toolkit/tree/master/packages/utils
  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  // IPC test
  ipcMain.on("ping", () => console.log("pong"));

  createWindow();

  app.on("activate", function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("before-quit", () => {
  closeDb();
});

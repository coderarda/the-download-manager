// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { contextBridge, ipcRenderer } from "electron";
import type { IUtils, URLEventCallback } from "../types";

const utilsObj: IUtils = {
    onUrl: (cb) => ipcRenderer.on("url", cb),
    removeOnURL: (event: URLEventCallback) => ipcRenderer.removeListener("url", event),
};

contextBridge.exposeInMainWorld("utils", utilsObj);
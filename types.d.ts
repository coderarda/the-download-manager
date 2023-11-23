import type { IpcRendererEvent } from "electron"

interface DownloadURLObj {
    id: number,
    url: string,
    title: string,
    filesize: number,
}

type ServerData = {
    reference: DownloadURLObj,
    update?: DownloadURLObj | number,
}

type URLEventCallback = (event: IpcRendererEvent, ...args: unknown[]) => void

export interface IUtils {
    onUrl: (cb: URLEventCallback) => void;
    removeOnURL: (cb: URLEventCallback) => void;
}

declare global {
    interface Window {
        utils: IUtils;
    }
}
export type DownloadURLObj = {
    url: string,
    title: string,
    filesize: number,
};

export interface IUtils {
    onUrl: (cb: (event: IpcRendererEvent, ...args: unknown[]) => void) => void;
    getSizeInfo: (cb: (event: IpcRendererEvent, ...args: unknown[]) => void) => void;
}

export type DataObj = {
    type: string,
    data: unknown,
}

declare global {
    interface Window {
        utils: IUtils;
    }
}
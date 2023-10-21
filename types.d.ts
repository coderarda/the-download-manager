export type DownloadURLObj = {
    url: string,
    title: string,
    filesize: number,
};

export interface IUtils {
    onUrl: (cb: (event: IpcRendererEvent, ...args: unknown[]) => void) => void;
}

declare global {
    interface Window {
        utils: IUtils;
    }
}
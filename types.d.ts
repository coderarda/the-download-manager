export type DownloadURLObj = {
    url: string,
};

export interface IUtils {
    onUrl: (cb: (event: IpcRendererEvent, ...args: unknown[]) => void) => void;
}

declare global {
    interface Window {
        utils: IUtils;
    }
}
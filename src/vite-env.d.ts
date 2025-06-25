/// <reference types="vite/client" />

declare type DownloadObj = {
    id: number,
    url: string,
    title: string,
    filesize: number,
};

declare type DownloadInfo = {
    id: number,
    chunk_size: number,
}

declare type DownloadStatus = {
    item: DownloadObj,
    paused: boolean,
    downloading: boolean,
}
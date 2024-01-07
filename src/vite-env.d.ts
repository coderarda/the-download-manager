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
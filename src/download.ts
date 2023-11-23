import { createWriteStream } from "fs";
import https from "https";
import { join } from "path";
import { parentPort, workerData } from "worker_threads";

const data = workerData;
const file = createWriteStream(join(".", data.title));
let download_size = 0;
https.get(data.url, (resp) => {
    resp.pipe(file);
    resp.on("data", (chunk: Uint8Array) => {
        download_size += chunk.byteLength;
        parentPort.postMessage(download_size);
    });
});
file.on("finish", () => {
    file.close();
    console.log("File downloaded.");
});
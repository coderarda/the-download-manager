import Express from "express";
import cors from "cors";
import type { DownloadURLObj, DataObj } from "../types";
import { createWriteStream } from "fs";
import https from "https";
import { join } from "path";

const app = Express();

app.listen(4000, () => console.log("Listening on port 4000."));

app.use(cors());
app.use(Express.json());

app.post("/url", (req, res) => {
    const data = req.body as DownloadURLObj;
    const urlObj: DataObj = {
        type: "url",
        data: data,
    }
    let download_size = 0;
    process.parentPort.postMessage(urlObj);
    res.status(200).send("Data received!");
    const file = createWriteStream(join(".", data.title));
    
    https.get(data.url, (resp) => {
        resp.pipe(file);
        resp.on("data", (chunk: Uint8Array) => {
            download_size += chunk.byteLength;
            const dataObj: DataObj = {
                type: "size",
                data: download_size,
            };
            process.parentPort.postMessage(dataObj);
        });
    });
    file.on("finish", () => {
        file.close();
        console.log("File downloaded.");
    });
});




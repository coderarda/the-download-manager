import Express from "express";
import cors from "cors";
import type { DownloadURLObj, ServerData } from "../types";
import { Worker } from "worker_threads";
import { join } from "path";

const DOWNLOAD_THREAD_PATH = join(__dirname, "./download.js");

const app = Express();

app.listen(4000, () => console.log("Listening on port 4000."));

app.use(cors());
app.use(Express.json());

let prev_id = 0;

app.post("/url", (req, res) => {
    const body = req.body as DownloadURLObj;
    body.id = prev_id;
    const data: ServerData = {
        reference: body,
        update: body,
    }
    prev_id++;
    process.parentPort.postMessage(data);
    const thread = new Worker(DOWNLOAD_THREAD_PATH, { workerData: data.reference });
    thread.on("message", (msg: number) => {
        const sData: ServerData = {
            reference: data.reference,
            update: msg,
        }
        process.parentPort.postMessage(sData);
    });
    res.status(200).send("Data received!");
});




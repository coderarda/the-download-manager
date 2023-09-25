import Express from "express";
import cors from "cors";

const app = Express();
app.use(cors());
app.use(Express.json());

app.listen(4000, () => console.log("Listening on port 4000."));

app.post("/url", (req, res) => {
    process.parentPort.postMessage(req.body);
    res.status(200);
    res.send(JSON.stringify("data received."));
});

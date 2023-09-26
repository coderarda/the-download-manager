import { List, ListItemText } from "@mui/material";
import { ipcRenderer } from "electron";
import React, { useEffect, useState } from "react";
import { DownloadURLObj } from "../../shared/types";

export function HomePage() {
    const [items, setItems] = useState<string[]>([]);
    useEffect(() => {
        ipcRenderer.postMessage("url", "Waiting for URLs.");
        ipcRenderer.on("url", (ev, reply: DownloadURLObj) => {
            setItems([...items, reply.url]);
        });
    }, []);
    return (
        <>
            <List>
                {items.map((el, i) => <ListItemText key={i}>{el}</ListItemText>)}
            </List>
        </>
    );
}
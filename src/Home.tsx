import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableRow,
} from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { Download } from "./Download";

export function Home() {
    const stored = sessionStorage.getItem("items");
    const [downloads, setDownloads] = useState<DownloadObj[]>(
        stored != null 
        ? JSON.parse(stored) as DownloadObj[] 
        : []
    );
    useEffect(() => {
        const unlisten = listen("ondownload", (e) => {
            let exists = false;
            const data = JSON.parse(e.payload as string) as DownloadObj;
            downloads.forEach((val) => {
                if(val.id == data.id) {
                    exists = true
                } 
            })
            if(!exists) {
                setDownloads([...downloads, data]);
            }
            sessionStorage.setItem("items", JSON.stringify(downloads));
        });
        return () => {
            unlisten.then((f) => f());
        };
    }, [downloads]);
    return (
        <Table aria-label="Downloads Table">
            <TableHead>
                <TableRow>
                    <TableCell>#</TableCell>
                    <TableCell>Filename</TableCell>
                    <TableCell>URL</TableCell>
                    <TableCell>Size</TableCell>
                    <TableCell>Percentage</TableCell>
                </TableRow>
            </TableHead>
            <TableBody>
                {downloads.map((val) => {
                    return <Download val={val} />
                })}
            </TableBody>
        </Table>
    );
}

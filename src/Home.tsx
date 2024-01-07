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
    const [downloads, setDownloads] = useState<DownloadObj[]>([]);
    useEffect(() => {
            const unlisten = listen("ondownload", (e) => {
                const data = JSON.parse(e.payload as string) as DownloadObj;
                setDownloads([...downloads, data]);
            });
        return () => {
            unlisten.then((f) => f());
        };
    }, []);
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

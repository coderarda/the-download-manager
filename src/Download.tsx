import { Pause, PlayArrow } from "@mui/icons-material";
import { IconButton, TableCell, TableRow } from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";

export function Download({ val }: { val: DownloadObj }) {
    const [percentage, setPercentage] = useState<number>(0);
    useEffect(() => {
        const f = listen("ondownloadupdate", (e) => {
            const data = JSON.parse(e.payload as string) as DownloadInfo;
            if(data.id == val.id) {
                setPercentage(data.chunk_size);
            }
        });
        return () => {
            f.then((fun) => fun());
        }
    }, []);
    return (
        <TableRow>
            <TableCell>{val.id}</TableCell>
            <TableCell>{val.title}</TableCell>
            <TableCell>{val.url}</TableCell>
            <TableCell>{(val.filesize / (1024 * 1024)).toFixed()} MB</TableCell>
            <TableCell>
                % {((percentage / val.filesize) * 100).toFixed()}
            </TableCell>
            <TableCell>
                <IconButton onClick={async () => await invoke("pause_download", { id: val.id })}>
                    <Pause />
                </IconButton>
                <IconButton onClick={async () => await invoke("resume", { id: val.id })}>
                    <PlayArrow />
                </IconButton>
            </TableCell>
        </TableRow>
    );
}
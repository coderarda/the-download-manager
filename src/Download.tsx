import { TableCell, TableRow } from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export function Download({ val }: { val: DownloadObj }) {
    const [percentage, setPercentage] = useState<number>(0);
    useEffect(() => {
        const f = listen("ondownloadupdate", (e) => {
            const data = JSON.parse(e.payload as string) as DownloadInfo;
            if(data.id == val.id) {
                setPercentage((p) => p + data.chunk_size);
            }
        });
        return () => {
            f.then((fun) => fun());
        }
    }, []);
    return (
        <TableRow key={val.id}>
            <TableCell>{val.id}</TableCell>
            <TableCell>{val.title}</TableCell>
            <TableCell>{val.url}</TableCell>
            <TableCell>{(val.filesize / (1024 * 1024)).toFixed() + " MB"}</TableCell>
            <TableCell>
                % {((percentage / val.filesize) * 100).toFixed()}
            </TableCell>
        </TableRow>
    );
}
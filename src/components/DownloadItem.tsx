import { TableRow, TableCell } from "@mui/material";
import React, { useEffect, useState } from "react";
import type { DownloadURLObj, ServerData } from "../../types";

export function DownloadItem(props: DownloadURLObj) {
    const [downloadPercentage, setDownloadPercentage] = useState<number>();
    useEffect(() => {
        window.utils.onUrl((e, data: ServerData) => {
            if(typeof data.update != "object" && data.reference.id == props.id) {
                setDownloadPercentage(data.update);
            }
        });
    }, []);
    return (
        <TableRow>
            <TableCell>{props.title}</TableCell>
            <TableCell>{props.url}</TableCell>
            <TableCell>%{((downloadPercentage / props.filesize) * 100).toFixed()}</TableCell>
            <TableCell>{(props.filesize / (1024 * 1024)).toFixed(2)} MB</TableCell>
        </TableRow>
    );
}
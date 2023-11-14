import { TableRow, TableCell } from "@mui/material";
import React, { useEffect, useState } from "react";
import type { DownloadURLObj } from "../../types";

export function DownloadItem(props: DownloadURLObj) {
    const [downloadedSize, setDownloadedSize] = useState<number>();
    useEffect(() => {
        window.utils.getSizeInfo((_, data) => {
            const result = data as number;
            setDownloadedSize(result);
        });
    }, []);
    
    return (
        <TableRow>
            <TableCell>{props.title}</TableCell>
            <TableCell>{props.url}</TableCell>
            <TableCell>%{((downloadedSize / props.filesize) * 100).toFixed()}</TableCell>
            <TableCell>{(props.filesize / (1024 * 1024)).toFixed(2)} MB</TableCell>
        </TableRow>
    );
}
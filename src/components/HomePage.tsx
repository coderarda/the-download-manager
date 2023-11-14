import { Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from "@mui/material";
import React, { useEffect, useState } from "react";
import type { DownloadURLObj } from "../../types";
import { DownloadItem } from "./DownloadItem";

export function HomePage() {
    const [items, setItems] = useState<DownloadURLObj[]>([]);
    useEffect(() => {        
        window.utils.onUrl((_, url: DownloadURLObj) => {
            setItems([...items, url]);
        });
    }, []);
    return (
        <TableContainer>
            <Table>
                <TableHead>
                    <TableRow>
                        <TableCell>Title</TableCell>
                        <TableCell>URL</TableCell>
                        <TableCell>Progress</TableCell>
                        <TableCell>Size</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {items.map((el, i) => {
                        return <DownloadItem key={i} {...el}></DownloadItem>
                    })}
                </TableBody>
            </Table>
        </TableContainer>
    );
}
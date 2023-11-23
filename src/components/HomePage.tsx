import { Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from "@mui/material";
import React, { useEffect, useState } from "react";
import type { DownloadURLObj, ServerData } from "../../types";
import { DownloadItem } from "./DownloadItem";

export function HomePage() {
    const stored = JSON.parse(sessionStorage.getItem("itemsObj"))?.items as DownloadURLObj[] || [];
    const [items, setItems] = useState<DownloadURLObj[]>(stored);   
    useEffect(() => {
        window.utils.onUrl((_, data: ServerData) => {
            if(data.update != null && typeof data.update == "object") {
                setItems([...items, data.update]);
            }
        });
        sessionStorage.setItem("itemsObj", JSON.stringify({ items }));
    }, [items]);
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
                    {items.map((el, i) => <DownloadItem key={i} {...el}></DownloadItem>)}
                </TableBody>
            </Table>
        </TableContainer>
    );
}
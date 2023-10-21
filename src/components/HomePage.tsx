import { Table, TableBody, TableCell, TableContainer, TableHead, TableRow } from "@mui/material";
import React, { useEffect, useState } from "react";
import type { DownloadURLObj } from "../../types";

export function HomePage() {
    const [items, setItems] = useState<DownloadURLObj[]>([]);
    
    useEffect(() => {
        window.utils.onUrl((_, msg: DownloadURLObj) => {
            setItems([...items, msg]);
        })
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
                        return (
                            <TableRow key={i}>
                                <TableCell>{el.title}</TableCell>
                                <TableCell>{el.url}</TableCell>
                                <TableCell>%{0 / el.filesize}</TableCell>
                                <TableCell>{el.filesize}</TableCell>
                            </TableRow>
                        );
                    })}
                </TableBody>
            </Table>
        </TableContainer>
    );
}
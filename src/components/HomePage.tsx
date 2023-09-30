import { List, ListItemText } from "@mui/material";
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
        <>
            <List>
                {items.map((el, i) => <ListItemText key={i}>{el.url}</ListItemText>)}
            </List>
        </>
    );
}
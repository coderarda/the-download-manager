import { AppBar, Toolbar, IconButton, Typography, Drawer, List, ListItemButton } from "@mui/material";
import { HomeOutlined, Menu } from "@mui/icons-material"
import { useState } from "react";
import { useNavigate } from "react-router-dom";

export function AppBarComponent() {
    const [title, setTitle] = useState("Home");
    const [open, setOpen] = useState(false);
    const navigate = useNavigate();

    return (
        <AppBar position="static">
            <Toolbar variant="dense">
                <IconButton edge="start" onClick={() => setOpen((val) => !val)}>
                    <Menu />
                </IconButton>
                <Drawer open={open} variant="temporary" anchor="left" onClose={() => setOpen(false)}>
                    <List>
                        <ListItemButton onClick={() => {
                            navigate("/");
                            setTitle("Home");
                            setOpen(!open);
                        }}>
                            <HomeOutlined />
                            <Typography sx={{ marginLeft: 1 }}>Home</Typography>
                        </ListItemButton>
                    </List>
                </Drawer>
                <Typography marginLeft={1}>{title}</Typography>
            </Toolbar>
        </AppBar>
    );

}
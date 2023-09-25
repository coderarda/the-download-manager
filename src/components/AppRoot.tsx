import {
    AppBar,
    CssBaseline,
    Drawer,
    IconButton,
    List,
    ListItemButton,
    ListItemIcon,
    ListItemText,
    Toolbar,
    Typography,
} from "@mui/material";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import React, { useState } from "react";
import { green } from "@mui/material/colors";
import MenuIcon from "@mui/icons-material/Menu";
import BackIcon from "@mui/icons-material/ArrowBack";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/700.css";
import "@fontsource/roboto/300.css";
import {
    Link,
    MemoryRouter,
    Routes,
    Route,
    useNavigate,
} from "react-router-dom";
import { HomePage } from "./HomePage";

const theme = createTheme({
    palette: {
        primary: {
            main: green[700],
        },
    },
    typography: {},
});

function Navbar() {
    const [isOpen, setIsOpen] = useState(false);
    const navigate = useNavigate();
    return (
        <AppBar position="static">
            <Drawer open={isOpen} sx={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                <List>
                    <ListItemButton alignItems="center" onClick={() => setIsOpen(false)}>
                        <ListItemIcon>
                            <BackIcon />
                        </ListItemIcon>
                        <ListItemText primary="Go Back"/>
                    </ListItemButton>
                    <ListItemButton>
                        <Link
                            style={{ textDecoration: "none", color: "initial" }}
                            onClick={() => setIsOpen(false)}
                            to="/downloads"
                        >
                        <ListItemText primary="Downloads" />
                        </Link>
                    </ListItemButton>
                </List>
            </Drawer>
            <Toolbar variant="dense">
                <IconButton onClick={() => navigate(-1)}>
                    <BackIcon />
                </IconButton>
                <IconButton onClick={() => setIsOpen(true)}>
                    <MenuIcon />
                </IconButton>
                <Typography variant="body1" marginLeft={1} component="div">
                    Home
                </Typography>
            </Toolbar>
        </AppBar>
    );
}

export function AppRoot() {
    return (
        <ThemeProvider theme={theme}>
            <MemoryRouter>
                <Navbar />
                <Routes>
                    <Route index={true} element={<HomePage />} />
                    <Route path="/downloads" element={<div>Downloads</div>} />
                </Routes>
                <CssBaseline />
            </MemoryRouter>
        </ThemeProvider>
    );
}

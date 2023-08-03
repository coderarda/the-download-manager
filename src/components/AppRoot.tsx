import { AppBar, CssBaseline, Drawer, IconButton, List, ListItem, Toolbar, Typography } from "@mui/material";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import React, { useState } from "react";
import { green } from "@mui/material/colors";
import MenuIcon from "@mui/icons-material/Menu";
import BackIcon from "@mui/icons-material/ArrowBack";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/100.css";
import { Link, MemoryRouter, Routes, Route, useNavigate } from "react-router-dom";
import { HomePage } from "./HomePage";

const theme = createTheme({
    palette: {
        mode: "dark",
        primary: {
            main: green[700],
        }
    },
    typography: {
        fontFamily: "'Roboto'",
        fontWeightBold: "bold",
        fontWeightLight: "lighter",
        fontWeightRegular: "normal",
        allVariants: {
            color: "white",
        }
    }
});

function Navbar() {
    const [isOpen, setIsOpen] = useState(false);
    const navigate = useNavigate();
    return (
        <AppBar position="static">
            <Drawer open={isOpen}>
                <IconButton onClick={() => setIsOpen(false)}>
                    <BackIcon />
                </IconButton>
                <List>
                    <ListItem>
                        <Link style={{ textDecoration: "none", color: "initial" }} onClick={() => setIsOpen(false)} to="/downloads">
                            <Typography>Downloads</Typography>
                        </Link>
                    </ListItem>
                </List>
            </Drawer>
            <Toolbar variant="dense">
                <IconButton onClick={() => navigate(-1)}>
                    <BackIcon />
                </IconButton>
                <IconButton onClick={() => setIsOpen(true)}>
                    <MenuIcon />
                </IconButton>
                <Typography variant="body1" marginLeft={1} component="div">Home</Typography>
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
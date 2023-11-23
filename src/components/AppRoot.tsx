import {
    AppBar,
    Box,
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
    useNavigate,
    RouterProvider,
    Outlet,
    createHashRouter,
    Navigate,
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

const router = createHashRouter([
    {
        path: "/",
        element: <Navbar />,
        children: [
            { index: true, element: <Navigate to="/home" replace />},
            { path: "home", element: <HomePage /> },
            { path: "settings", element: <Typography>Settings</Typography> },
        ]
    },
]);

function Navbar() {
    const [isOpen, setIsOpen] = useState(false);
    const navigate = useNavigate();
    // Fix the link buttons
    return (
        <Box>
            <AppBar position="static">
                <Drawer open={isOpen} sx={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <List>
                        <ListItemButton alignItems="center" onClick={() => setIsOpen(false)}>
                            <ListItemIcon>
                                <BackIcon />
                            </ListItemIcon>
                            <ListItemText primary="Go Back" />
                        </ListItemButton>
                        <ListItemButton onClick={() => {
                            navigate("/home");
                            setIsOpen(false);
                        }}>
                            <ListItemText primary="Home" />
                        </ListItemButton>
                        <ListItemButton onClick={() => {
                            navigate("/settings");
                            setIsOpen(false);
                        }}>
                            <ListItemText primary="Settings" />
                        </ListItemButton>
                    </List>
                </Drawer>
                <Toolbar variant="dense">
                    <IconButton onClick={() => {
                        navigate(-1);
                    }}>
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
            <Outlet />
        </Box>
    );
}

export function AppRoot() {
    return (
        <ThemeProvider theme={theme}>
            <RouterProvider router={router} />
            <CssBaseline />
        </ThemeProvider>
    );
}

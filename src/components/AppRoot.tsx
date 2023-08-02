import { AppBar, Box, Card, CardContent, CssBaseline, IconButton, Toolbar, Typography } from "@mui/material";
import React from "react";
import { ThemeProvider, createTheme } from "@mui/material/styles";
import { green } from "@mui/material/colors";
import MenuIcon from "@mui/icons-material/Menu";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/100.css";

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

export function AppRoot() {
    return (
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <AppBar>
                <Toolbar variant="dense">
                    <IconButton>
                        <MenuIcon />
                    </IconButton>
                    <Typography variant="body1" marginLeft={1} component="div">Home</Typography>
                </Toolbar>
            </AppBar>
            <Box mt={6} p={3}>
                <Card>
                    <CardContent>
                        <Typography>This is a test</Typography>
                    </CardContent>
                </Card>
            </Box>
        </ThemeProvider>
    );
}
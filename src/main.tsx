import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, MemoryRouter, Outlet, Route, Routes, useLocation } from "react-router-dom";
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';
import { createTheme, CssBaseline, ThemeProvider } from "@mui/material";
import { green, blue, grey, common, lightBlue, purple } from "@mui/material/colors";
import { AppBarComponent } from "./AppBarComponent";
import { Home } from "./Home";
import { Settings } from "./Settings";

const theme = createTheme({
    palette: {
        primary: lightBlue,
        secondary: purple,
        background: {
            default: grey[100],
        },
        mode: "light",
    }
});

function Root() {
    return (
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <BrowserRouter>
                <AppBarComponent/>
                <Routes>
                    <Route path="/" element={<Home />} />
                    <Route path="/settings" element={<Settings />} />
                </Routes>
            </BrowserRouter>
        </ThemeProvider>
    );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <Root />
    </React.StrictMode>
);

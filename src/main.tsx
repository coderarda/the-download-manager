import React, { Suspense } from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import "@fontsource/roboto/300.css";
import "@fontsource/roboto/400.css";
import "@fontsource/roboto/500.css";
import "@fontsource/roboto/700.css";
import { createTheme, ThemeProvider } from "@mui/material/styles";
import { Box, Button, CircularProgress, CssBaseline } from "@mui/material";
import { common, deepPurple } from "@mui/material/colors";
import { Home } from "./Home";
import { Settings } from "./Settings";
import { AppBarComponent } from "./AppBarComponent";
import { Close, Crop75, Minimize } from "@mui/icons-material";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { platform } from "@tauri-apps/plugin-os";
import { AppProvider } from "./AppContext";

const theme = createTheme({
    palette: {
        text: {
            primary: common.black,
        },
        primary: {
            main: deepPurple[400],
        },
        secondary: {
            main: common.white,
        }
    },
});

// const initialDownloads: DownloadStatus[] = [];

/*(async () => {
    const arr = await invoke<DownloadStatus[]>("load_downloads");
    initialDownloads.push(...arr);
})();*/

function Root() {
    const win = getCurrentWindow();
    return (
        <ThemeProvider theme={theme}>
            <CssBaseline/>
            <Box data-tauri-drag-region sx={{background: deepPurple[500]}} width={"100%"} display={"flex"}
                 flexDirection={"row"} justifyContent={"end"}>
                <Button onClick={() => {
                    (async () => {
                        await win.minimize();
                    })();
                }}><Minimize color="secondary"/></Button>
                <Button onClick={async () => {
                    // check if macOS then fullscreen
                    if (platform() == "macos") {
                        if (await win.isFullscreen()) {
                            await win.setFullscreen(false);
                        } else {
                            await win.setFullscreen(true);
                        }
                    } else {
                        await win.maximize();
                    }
                }}><Crop75 color="secondary"/></Button>
                <Button onClick={() => {
                    (async () => {
                        await win.close();
                    })()
                }}><Close color="secondary"/></Button>
            </Box>
            <BrowserRouter>
                <Suspense fallback={<CircularProgress/>}>
                    <AppBarComponent/>
                </Suspense>
                <Routes>
                    <Route index element={<Home/>}/>
                    <Route path="settings" element={<Settings/>}/>
                </Routes>
            </BrowserRouter>
        </ThemeProvider>
    );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
        <AppProvider>
            <Root/>
        </AppProvider>
    </React.StrictMode>
);

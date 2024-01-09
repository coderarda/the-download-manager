import "./App.css";
import { Outlet, Route, Routes, useNavigate } from "react-router-dom";
import { Home } from "./Home";
import { ThemeProvider, createTheme } from "@mui/material";
import { blue, green, grey } from "@mui/material/colors";
import { Settings } from "./Settings";

function App() {
    const theme = createTheme({
        palette: {
            primary: green,
            secondary: blue,
            background: {
                default: grey[50],
            }
        }
    });

    return (
        <ThemeProvider theme={theme}>
            <Routes>
                <Route path="/" element={<Home />} />
                <Route path="/settings" element={<Settings />} />
            </Routes>
            <Outlet />
        </ThemeProvider>
    );
}

export default App;

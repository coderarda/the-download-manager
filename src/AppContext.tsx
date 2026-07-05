import React, { createContext, useContext, useState, ReactNode, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

// Define types for your state
interface AppState {
    downloads: DownloadStatus[];
    setDownloads: React.Dispatch<React.SetStateAction<DownloadStatus[]>>;
}

// Create the context
const AppContext = createContext<AppState | undefined>(undefined);

// Create a provider component
export const AppProvider = ({ children }: { children: ReactNode }) => {
    const [downloads, setDownloads] = useState<DownloadStatus[]>([]);

    useEffect(() => {
        const unlisten = listen("downloadfirstupdate", (e) => {
            const data = e.payload as DownloadStatus;
            downloads.forEach((d) => {
                if (d.item.id == data.item.id) {
                    let idx = downloads.indexOf(d);
                    downloads[idx] = data;
                    setDownloads([...downloads]);
                }
            })
        })
        return () => {
            unlisten.then((fun) => fun()).catch((err) => console.log(err));
        }
    }, []);


    return (
        <AppContext.Provider value={{ downloads, setDownloads }}>
            {children}
        </AppContext.Provider>
    );
};

// Custom hook to use the context
export const useAppContext = () => {
    const context = useContext(AppContext);
    if (!context) {
        throw new Error("useAppContext must be used within an AppProvider");
    }
    return context;
};
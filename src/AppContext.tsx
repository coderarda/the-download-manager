import React, { createContext, useContext, useState, ReactNode } from "react";

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
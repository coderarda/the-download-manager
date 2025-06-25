import {
    Box,
    Button,
    Modal,
    SpeedDial,
    SpeedDialAction,
    SpeedDialIcon,
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableRow,
    TextField,
    Typography,
} from "@mui/material";
import {listen} from "@tauri-apps/api/event";
import React, {useState, useEffect} from "react";
import Download from "./Download";
import {Add, Settings, Alarm} from "@mui/icons-material";
import {useNavigate} from "react-router-dom";
import {invoke} from "@tauri-apps/api/core";
import ScheduleDownloadModal from "./ScheduleDownloadModal";

const style = {
    position: "absolute",
    top: "50%",
    left: "50%",
    transform: "translate(-50%, -50%)",
    width: 400,
    bgcolor: "background.paper",
    boxShadow: 24,
    p: 4,
};

export function Home({initial}: { initial: DownloadStatus[] }) {
    const [openDial, setOpenDial] = useState(false);
    const [openAddLink, setOpenAddLink] = useState(false);
    const [openAutoAddLink, setOpenAutoAddLink] = useState(false);
    const [currURL, setCurrURL] = useState<string>();
    const [scheduleDownloadModalVisibility, setScheduleDownloadModalVisibility] =
        useState(false);
    const [filename, setFilename] = useState<string>();
    const [downloads, setDownloads] = useState<DownloadStatus[]>(initial);
    const [queuedDownload, setQueuedDownload] = useState<DownloadStatus | null>(null);
    useEffect(() => {
        // 1. load initial downloads
        /* (async () => {
            const initial = await invoke<DownloadStatus[]>("load_downloads");
            setDownloads(prev => [...prev, ...initial]);
        })();
        */
        // 2. ondownload listener
        const unlisten = listen<DownloadStatus>("ondownload", e => {
            const download = e.payload;
            console.log("Received download payload:", JSON.stringify(e.payload));
            const exists = downloads.some(d => d.item.id === download.item.id);
            if (!exists) {
                setOpenAutoAddLink(true);
                setCurrURL(download.item.url);
                setFilename(download.item.title);
                setQueuedDownload(download);
            }
        });

        // 3. download_removed listener
        const unlistenRemoved = listen<number>("download_removed", e => {
            setDownloads(prev => prev.filter(d => d.item.id !== e.payload));
        });
        sessionStorage.setItem("items", JSON.stringify(downloads));
        // 4. clean-up
        return () => {
            unlisten.then(f => f()).catch(console.error);
            unlistenRemoved.then(f => f()).catch(console.error);
        };
    }, []);      // still only run once on mount
    const navigate = useNavigate();
    return (
        <>
            <Modal
                open={openAutoAddLink}
                onClose={() => setOpenAutoAddLink(false)}
                aria-labelledby="modal-modal-title"
            >
                <Box sx={style}>
                    <Typography
                        marginY={2}
                        marginX={1}
                        id="modal-modal-title"
                        fontWeight={"bold"}
                        variant="h6"
                    >
                        Add Download
                    </Typography>
                    <TextField
                        label="URL"
                        size="small"
                        margin="normal"
                        fullWidth
                        value={currURL}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                            setCurrURL(e.target.value);
                        }}
                    />
                    <TextField
                        label="Filename"
                        size="small"
                        margin="normal"
                        fullWidth
                        value={filename}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                            setFilename(e.target.value);
                        }}
                    />
                    <Box flexDirection={"row"} paddingTop={2}>
                        <Button
                            variant="contained"
                            sx={{marginRight: 2, position: "relative"}}
                            color="primary"
                            onClick={() => {
                                setOpenAutoAddLink(false);
                                if (queuedDownload) {
                                    setDownloads([...downloads, queuedDownload]);
                                    invoke("download", {download: queuedDownload})
                                        .then((value) => {
                                            console.log(value);
                                        })
                                        .catch((err) => {
                                            console.error(err);
                                        });
                                }
                            }}
                        >
                            Download
                        </Button>
                        <Button
                            variant="contained"
                            color="error"
                            onClick={() => {
                                setOpenAutoAddLink(false);
                            }}
                        >
                            Cancel
                        </Button>
                    </Box>
                </Box>
            </Modal>
            <Modal
                open={openAddLink}
                onClose={() => setOpenAddLink(false)}
                aria-labelledby="modal-modal-title"
            >
                <Box sx={style}>
                    <Typography
                        marginY={2}
                        marginX={1}
                        id="modal-modal-title"
                        fontWeight={"bold"}
                        variant="h6"
                    >
                        Add Download
                    </Typography>
                    <TextField
                        label="URL"
                        id="url-box"
                        size="small"
                        margin="normal"
                        fullWidth
                        value={currURL}
                        onChange={(e: React.ChangeEvent<HTMLInputElement>) => {
                            setCurrURL(e.target.value);
                        }}
                    />
                    <Box flexDirection={"row"} paddingTop={2}>
                        <Button
                            variant="contained"
                            sx={{marginRight: 2, position: "relative"}}
                            color="primary"
                            id="download-btn"
                            onClick={() => {
                                setOpenAddLink(false);
                                console.log("Link added!");
                                if (currURL != null) {
                                    (async () => {
                                        const obj: DownloadObj = await invoke("get_download_info", {
                                            url: currURL,
                                        });
                                        const newStatus: DownloadStatus = {
                                            item: obj,
                                            paused: false,
                                            downloading: false,
                                        };
                                        console.log("Download info function invoked!");
                                        setDownloads([...downloads, newStatus]);
                                        setFilename(obj.title);
                                        console.log(`Download of filename ${obj.title} started!`);
                                        await invoke("download_manually_from_url", {download: obj});
                                    })();
                                }
                            }}
                        >
                            Download
                        </Button>
                        <Button
                            variant="contained"
                            color="error"
                            onClick={() => {
                                setOpenAddLink(false);
                            }}
                        >
                            Cancel
                        </Button>
                    </Box>
                </Box>
            </Modal>
            <ScheduleDownloadModal
                open={scheduleDownloadModalVisibility}
                handleClose={() => {
                    setScheduleDownloadModalVisibility(false);
                }}
            />
            <Table aria-label="Downloads Table">
                <TableHead>
                    <TableRow>
                        <TableCell>#</TableCell>
                        <TableCell>Filename</TableCell>
                        <TableCell>URL</TableCell>
                        <TableCell>Size</TableCell>
                        <TableCell>Percentage</TableCell>
                        <TableCell>Actions</TableCell>
                    </TableRow>
                </TableHead>
                <TableBody>
                    {downloads.map((val: DownloadStatus) => {
                        const d: DownloadObj = val.item;
                        console.log(val.item);
                        return <Download key={d.id} status={val}/>;
                    })}
                </TableBody>
            </Table>
            <SpeedDial
                open={openDial}
                onOpen={() => setOpenDial(true)}
                onClose={() => setOpenDial(false)}
                ariaLabel="Actions"
                sx={{position: "absolute", right: 16, bottom: 16}}
                icon={<SpeedDialIcon/>}
            >
                <SpeedDialAction
                    key={0}
                    tooltipTitle="Download from URL"
                    tooltipOpen
                    icon={<Add/>}
                    onClick={() => {
                        setOpenDial(false);
                        setOpenAddLink(true);
                    }}
                    id="download-from-url"
                />
                <SpeedDialAction
                    key={1}
                    tooltipTitle="Schedule Download"
                    tooltipOpen
                    icon={<Alarm/>}
                    onClick={() => {
                        setOpenDial(false);
                        setScheduleDownloadModalVisibility(true);
                    }}
                />
                <SpeedDialAction
                    key={2}
                    tooltipTitle="Settings"
                    tooltipOpen
                    icon={<Settings/>}
                    onClick={() => {
                        setOpenDial(false);
                        navigate("/settings");
                    }}
                />
            </SpeedDial>
        </>
    );
}
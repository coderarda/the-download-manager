import {
    Box,
    Button,
    FormControl,
    Input,
    InputLabel,
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
import { listen } from "@tauri-apps/api/event";
import { useState, useEffect } from "react";
import { Download } from "./Download";
import { Add, Settings } from "@mui/icons-material";
import { useNavigate } from "react-router-dom";

const style = {
    position: 'absolute' as 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    width: 400,
    bgcolor: 'background.paper',
    boxShadow: 24,
    p: 4,
};

export function Home() {
    const stored = sessionStorage.getItem("items");
    const [downloadable, setDownloadable] = useState<boolean>(false);
    const [currDownload, setCurrDownload] = useState<DownloadObj>();
    const [downloads, setDownloads] = useState<DownloadObj[]>(
        stored != null
            ? JSON.parse(stored) as DownloadObj[]
            : []
    );
    useEffect(() => {
        const unlisten = listen("ondownload", (e) => {
            let exists = false;
            const data = JSON.parse(e.payload as string) as DownloadObj;
            downloads.forEach((val) => {
                if (val.id == data.id) {
                    exists = true
                }
            });
            if (!exists) {
                // Fix the download removal and registering part.
                setCurrDownload(data);
                setOpenAutoAddLink(true);
                setDownloads([...downloads, data]);
            }
            sessionStorage.setItem("items", JSON.stringify(downloads));
        });
        return () => {
            unlisten.then((f) => f());
        };
    }, [downloads]);
    const [openDial, setOpenDial] = useState(false);
    const [openAddLink, setOpenAddLink] = useState(false);
    const [openAutoAddLink, setOpenAutoAddLink] = useState(false);
    const navigate = useNavigate();
    return (
        <>
            <Modal
                open={openAutoAddLink}
                onClose={() => setOpenAutoAddLink(false)}
                aria-labelledby="modal-modal-title">
                <Box sx={style}>
                    <Typography id="modal-modal-title" fontWeight={"bold"} variant="h6">Add Download</Typography>
                    <TextField label="URL" size="small" fullWidth value={currDownload?.url} />
                    <Button variant="contained" color="primary" onClick={() => {
                        setDownloadable(true);
                        setOpenAutoAddLink(false);
                    }} fullWidth>Download</Button>
                    <Button variant="contained" color="error" onClick={() => {
                        setDownloadable(false);
                        setOpenAutoAddLink(false);
                    }} fullWidth>Cancel</Button>
                </Box>
            </Modal>
            <Modal 
                open={openAddLink}
                onClose={() => setOpenAddLink(false)}
                aria-labelledby="modal-modal-title"
            >
                <Box sx={style}>
                    <Typography id="modal-modal-title" fontWeight={"bold"} variant="h6">Schedule Download</Typography>
                </Box>
            </Modal>
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
                    {downloads.map((val) => {
                        return <Download val={val} />
                    })}
                </TableBody>
            </Table>
            <SpeedDial
                open={openDial}
                onOpen={() => setOpenDial(true)}
                onClose={() => setOpenDial(false)}
                ariaLabel="Actions"
                sx={{ position: "absolute", right: 16, bottom: 16 }}
                icon={<SpeedDialIcon />}>
                    <SpeedDialAction key={0} tooltipTitle="Schedule Download" tooltipOpen icon={<Add />} onClick={() => {
                        setOpenDial(false);
                        setOpenAddLink(true);
                    }}/>
                    <SpeedDialAction key={1} tooltipTitle="Settings" tooltipOpen icon={<Settings />} onClick={() => {
                        setOpenDial(false);
                        navigate("/settings");
                    }}/>
            </SpeedDial>
        </>
    );
}

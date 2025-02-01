import { Pause, PlayArrow } from "@mui/icons-material";
import { Box, Button, LinearProgress, TableCell, TableRow, Typography } from "@mui/material";
import { listen } from "@tauri-apps/api/event";
import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api";

export default function Download({ val }: { val: DownloadObj }) {
    const [percentage, setPercentage] = useState<number>(0);
    const [pausable, setPausable] = useState(true);
    const [isPaused, setIsPaused] = useState<boolean>(false);
    let completePercent = parseInt(((percentage / val.filesize) * 100).toFixed());
    useEffect(() => {
        const unlisten = listen("downloadpauseinfo", (e) => {
            const data = e.payload as boolean;
            if (data) {
                setPausable(true);
            } else {
                setPausable(false);
            }
        });
        const f = listen("ondownloadupdate", (e) => {
            const data = e.payload as DownloadInfo;
            if (data.id == val.id) {
                setPercentage(data.chunk_size);
            }
        });
        return () => {
            f.then((fun) => fun()).catch((err) => console.log(err));
            unlisten.then((fun) => fun()).catch((err) => console.log(err));
        };
    }, []);
    return (
        <TableRow>
            <TableCell>{val.id}</TableCell>
            <TableCell>{val.title}</TableCell>
            <TableCell>{val.url}</TableCell>
            <TableCell>{(val.filesize / (1024 * 1024)).toFixed()} MB</TableCell>
            <TableCell>
                {completePercent == 100 ? "Complete" : "%" + completePercent} 
                <LinearProgress variant="determinate" value={completePercent}/>
            </TableCell>
            <TableCell>
                <Box flexDirection="row" display="flex">
                    <Button
                        disabled={!pausable}
                        startIcon={isPaused ? <PlayArrow /> : <Pause />}
                        size="small"
                        variant="contained"
                        color="primary"
                        onClick={
                            async () => {
                                if(!isPaused) {
                                    await invoke("pause_download", { id: val.id });
                                    setIsPaused(true);
                                } else {
                                    await invoke("resume", { id: val.id });
                                    setIsPaused(false);
                                }
                            }
                        }
                    >
                        {isPaused ? <Typography>Resume</Typography> : <Typography>Pause</Typography>}
                    </Button>
                </Box>
            </TableCell>
        </TableRow>
    );
}

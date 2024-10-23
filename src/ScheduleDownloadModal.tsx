import React, { useEffect, useState } from 'react';
import { Modal, Box, TextField, Button, Typography } from '@mui/material';
import { TimePicker } from '@mui/x-date-pickers';
import { AdapterMoment } from '@mui/x-date-pickers/AdapterMoment';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import moment, { Moment } from 'moment';

const ScheduleDownloadModal = ({ open, handleClose }: { open: boolean, handleClose: () => any }) => {
    const time = moment();
    const [selectedTime, setSelectedTime] = useState<Moment>(time);
    const [link, setLink] = useState('');

    const handleTimeChange = (newTime: Moment | null) => {
        // Look up moment js and fix here
        if (newTime)
            setSelectedTime(newTime);
    };

    const handleLinkChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setLink(event.target.value);
    };

    const handleSubmit = () => {
        // Handle form submission logic here
        console.log('Scheduled Time:', selectedTime);
        console.log('Download Link:', link);
        handleClose();
    };

    return (
        <Modal open={open} onClose={handleClose}>
            <Box sx={{
                position: 'absolute',
                top: '50%',
                left: '50%',
                transform: 'translate(-50%, -50%)',
                width: 400,
                zIndex: 2,
                bgcolor: 'background.paper',
                boxShadow: 24,
                p: 4
            }}>
                <Typography variant="h6" component="h2">
                    Schedule Download
                </Typography>
                <LocalizationProvider dateAdapter={AdapterMoment}>
                    <TimePicker
                        label="Select Time"
                        value={selectedTime}
                        onChange={handleTimeChange}
                    />
                </LocalizationProvider>
                <TextField
                    label="Download Link"
                    value={link}
                    onChange={handleLinkChange}
                    fullWidth
                    margin="normal"
                />
                <Button variant="contained" color="primary" onClick={handleSubmit} fullWidth>
                    Schedule
                </Button>
            </Box>
        </Modal>
    );
};

export default ScheduleDownloadModal;

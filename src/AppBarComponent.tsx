import { ArrowBack } from "@mui/icons-material";
import { AppBar, IconButton, Toolbar, Typography } from "@mui/material";
import React, { useEffect, useState } from "react";
import { Link, LinkProps, useLocation } from "react-router-dom";

export function AppBarComponent() {
    const loc = useLocation();
    const [title, setTitle] = useState("");
    const LinkBehavior = React.forwardRef<HTMLAnchorElement, Omit<LinkProps, "to"> & { href: LinkProps['to'] }>((props, ref) => (
        <Link ref={ref} to={props.href} {...props} role={undefined} />
    ));
    useEffect(() => {
        switch(loc.pathname) {
            case "/": 
                setTitle("Home");
                break;
            case "/settings":
                setTitle("Settings");
                break;
            default:
                setTitle("Home");
        }
    }, [loc]);
    
    return (
        <AppBar position="relative" sx={{ zIndex: (theme) => theme.zIndex.drawer + 1 }}>
            <Toolbar variant="dense">
                <IconButton LinkComponent={LinkBehavior} href="..">
                    <ArrowBack />
                </IconButton>
                <Typography marginLeft={1}>{title}</Typography>
            </Toolbar>
        </AppBar>
    );

}
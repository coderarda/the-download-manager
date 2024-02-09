import { ArrowBack } from "@mui/icons-material";
import { AppBar, IconButton, Toolbar, Typography } from "@mui/material";
import React, { useState } from "react";
import { Link, LinkProps } from "react-router-dom";

export function AppBarComponent({ title }: { title: string }) {
    const LinkBehavior = React.forwardRef<HTMLAnchorElement, Omit<LinkProps, "to"> & { href: LinkProps['to'] }>((props, ref) => (
        <Link ref={ref} to={props.href} {...props} role={undefined} />
    ));
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
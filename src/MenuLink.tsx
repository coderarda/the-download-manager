import { MenuItem, Typography, Link as MuiLink } from "@mui/material";
import React from "react";
import { Link, LinkProps } from "react-router-dom";

export function MenuLink({ href, text, icon }: { href: string, text: string, icon: React.ReactNode }) {
    const LinkBehavior = React.forwardRef<HTMLAnchorElement, Omit<LinkProps, "to"> & { href: LinkProps['to'] }>((props, ref) => (
        <Link ref={ref} to={props.href} {...props} role={undefined} />
    ));
    return (
        <MenuItem>
            {icon}
            <MuiLink sx={{ textDecoration: "none", color: "initial" }} component={LinkBehavior} href={href}>
                <Typography paddingX={1}>{text}</Typography>
            </MuiLink>
        </MenuItem>
    );
}
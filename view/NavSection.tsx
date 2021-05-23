import { CommandBarButton, Nav, Persona, Stack } from "@fluentui/react"
import React from "react"
import { useHistory } from "react-router"
import { useAppSelector } from "./hooks/useApp"
import {json} from "./request"

const NavSection = () => {
    const history = useHistory();
    const active_user_state = useAppSelector(state => state.active_user);

    const logout = () => {
        json.post("/auth/logout",{}).then(() => {
            window.location.pathname = "/auth/login";
        }).catch(console.error)
    }

    let full_name = null;
    let username = "unknown";

    if (active_user_state.user) {
        username = active_user_state.user.username;
        full_name = active_user_state.user.full_name;
    }

    let nav_groups = [
        {
            name: "Home",
            links: [
                {
                    name: "Entries",
                    url: "/entries"
                },
                {
                    name: "Fields",
                    url: "/custom_fields"
                },
                {
                    name: "Tags",
                    url: "/tags"
                }
            ]
        },
        {
            name: "Manage",
            links: [
                {
                    name: "Users",
                    url: "/users"
                },
                {
                    name: "Account",
                    url: "/account"
                },
                {
                    name: "Settings",
                    url: "/settings"
                }
            ]
        }
    ];

    if (active_user_state.user.level === 1) {
        nav_groups.push({
            name: "Admin",
            links: [
                {
                    name: "Users",
                    url: "/admin/users"
                }
            ]
        });
    }

    return <Stack tokens={{padding: 4, childrenGap: 8}}>
        <Persona
            text={full_name ?? username}
            secondaryText={full_name != null ? username : null}
        />
        <Stack horizontal styles={{root: {height: 44}}}>
            <CommandBarButton
                text="Logout"
                iconProps={{iconName: "Leave"}}
                onClick={logout}
            />
        </Stack>
        <Nav
            onLinkClick={(e,i) => {
                e.preventDefault();
                history.push(i.url);
            }}
            groups={nav_groups}
        />
    </Stack>
}

export default NavSection;
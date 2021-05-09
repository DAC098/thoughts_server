import { CommandBarButton, Nav, Persona, Stack } from "@fluentui/react";
import React from "react"
import { useHistory } from "react-router";
import { useAppDispatch, useAppSelector } from "./hooks/useApp";
import { actions as active_user_actions } from "./redux/slices/active_user"
import { actions as entries_actions } from "./redux/slices/entries"
import { actions as mood_fields_actions } from "./redux/slices/mood_fields"
import {json} from "./request"

const NavSection = () => {
    const history = useHistory();
    const active_user_state = useAppSelector(state => state.active_user);
    const dispatch = useAppDispatch();

    const logout = () => {
        json.post("/auth/logout",{}).then(() => {
            dispatch(active_user_actions.clear_user());
            dispatch(entries_actions.clear_entries());
            dispatch(mood_fields_actions.clear_mood_fields());
            history.push("/auth/login");
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
                    url: "/mood_fields"
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
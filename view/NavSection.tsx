import { CommandBarButton, IconButton, Nav, Persona, Stack } from "@fluentui/react"
import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import React, { Reducer, useEffect, useReducer, useState } from "react"
import { useHistory } from "react-router"
import useAppDispatch from "./hooks/useAppDispatch"
import useAppSelector from "./hooks/useAppSelector"
import { view_actions } from "./redux/slices/view"
import { SliceActionTypes } from "./redux/types"
import {json} from "./request"
import resize_listener from "./util/ResizeListener"

const NavSection = () => {
    const nav_width = 200;
    const history = useHistory();
    const app_dispatch = useAppDispatch();
    const active_user_state = useAppSelector(state => state.active_user);
    const view_state = useAppSelector(state => state.view);

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
                },
                {
                    name: "Global Fields",
                    url: "/global/custom_fields"
                }
            ]
        });
    }

    return <div style={{
        width: view_state.is_min_width ?
               0 :
               (view_state.visible ? nav_width : 0)
    }}>
        <Stack
            tokens={{padding: 4, childrenGap: 8}}
            styles={{root: {
                position: "absolute",
                top: 0,
                left: view_state.visible ? 0 : `-${nav_width}px`,
                width: nav_width,
                height: "100%",
                zIndex: 2,
                backgroundColor: "white"
            }}}
        >
            {view_state.is_min_width ?
                <IconButton
                    iconProps={{iconName: "Cancel"}}
                    styles={{"root": {
                        position: "absolute",
                        top: 0,
                        right: 0,
                        zIndex: 2
                    }}}
                    onClick={() => app_dispatch(view_actions.set_visible(false))}
                />
                :
                null
            }
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
    </div>
}

export default NavSection;
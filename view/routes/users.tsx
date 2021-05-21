import { IconButton, Persona, Stack } from "@fluentui/react";
import React, { useEffect, useState } from "react"
import { Link, useHistory } from "react-router-dom";
import { UserListItemJson } from "../api/types"
import api from "../api";

const Users = () => {
    let [allowed, setAllowed] = useState<UserListItemJson[]>([]);
    let [given, setGiven] = useState<UserListItemJson[]>([]);
    let [loading, setLoading] = useState(false);

    const history = useHistory();

    const fetchUserList = () => {
        if (loading) {
            return;
        }

        setLoading(true);

        api.users.get().then(body => {
            setAllowed(body.allowed);
            setGiven(body.given);
        }).catch(console.error).then(() => {
            setLoading(false)
        });
    }

    useEffect(() => {
        fetchUserList();
    }, []);

    return <Stack horizontal tokens={{padding: 8, childrenGap: 8}}>
        <Stack tokens={{childrenGap: 8}}>
            <h4 style={{minWidth: 158}}>Allowed Access</h4>
            {allowed.map(v =>
                <Stack key={v.id} horizontal verticalAlign="center" tokens={{childrenGap: 8}}>
                    <Persona
                        text={v.full_name ?? v.username}
                        secondaryText={v.full_name != null ? v.username : null}
                        onRenderPrimaryText={() => <Link to={`/users/${v.id}`}>
                            {v.full_name ?? v.username}
                        </Link>}
                    />
                    <IconButton
                        menuProps={{
                            items: [
                                {
                                    key: "entries",
                                    text: "Entries",
                                    onClick: () => history.push(`/users/${v.id}/entries`)
                                },
                                {
                                    key: "mood_fields",
                                    text: "Fields",
                                    onClick: () => history.push(`/users/${v.id}/mood_fields`)
                                },
                                {
                                    key: "drop_access",
                                    text: "Drop Access",
                                    onClick: () => {}
                                }
                            ]
                        }}
                    />
                </Stack>
            )}
        </Stack>
        <Stack tokens={{childrenGap: 8}}>
            <h4 style={{minWidth: 158}}>Given Access</h4>
            {given.map(v => <Stack key={v.id} horizontal verticalAlign="center" tokens={{childrenGap: 8}}>
                <Persona
                    text={v.full_name ?? v.username}
                    secondaryText={v.full_name != null ? v.username : null}
                    onRenderPrimaryText={() => <Link to={`/users/${v.id}`}>
                        {v.full_name ?? v.username}
                    </Link>}
                />
                <IconButton
                    menuProps={{
                        items: [
                            {
                                key: "revoke",
                                text: "Revoke",
                                onClick: () => {}
                            }
                        ]
                    }}
                />
            </Stack>)}
        </Stack>
    </Stack>
}

export default Users;
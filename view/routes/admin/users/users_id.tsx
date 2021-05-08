import { DefaultButton, Dialog, DialogFooter, DialogType, Dropdown, IconButton, Persona, PersonaSize, ScrollablePane, Stack, TextField } from "@fluentui/react"
import React, { useEffect, useState } from "react"
import { useHistory, useLocation, useParams } from "react-router"
import { makeUserDataJson, UserDataJson } from "../../../api/types"
import IndentSection from "../../../components/IndentSection"
import * as api from "../../../api"
import { json } from "../../../request"

const UserInformation = () => {
    const params = useParams<{user_id: string}>();
    const history = useHistory();

    let [user, setUser] = useState<UserDataJson>(makeUserDataJson());
    let [password, setPassword] = useState("");
    let [sending, setSending] = useState(false);
    let [loading, setLoading] = useState(false);
    let [deleting, setDeleting] = useState(false);
    let [prep_delete, setPrepDelete] = useState(false);

    const sendUpdate = () => {
        if (sending) {
            return;
        }

        setSending(true);

        let promise = null;
        
        if (params.user_id === "0") {
            promise = api.admin.users.post({...user, password}).then(u => {
                setUser(u);
                history.push(`/admin/users/${u.id}`);
            });
        } else {
            promise = api.admin.users.id.put(params.user_id, user).then(u => {
                setUser(u);
            });
        }

        promise.catch(console.error).then(() => {
            setSending(false);
        });
    }

    const sendDelete = () => {
        if (deleting) {
            return;
        }

        if (params.user_id === "0") {
            return;
        }

        setDeleting(true);

        json.delete(`/admin/users/${params.user_id}`).then(() => {
            history.push("/admin/users");
        }).catch((e) => {
            console.error(e);
            setDeleting(false);
        });
    }

    const fetchUser = () => {
        if (loading) {
            return;
        }

        setLoading(true);

        api.admin.users.id.get(params.user_id).then(u => {
            setUser(u);
        }).catch(console.error).then(() => {
            setLoading(false);
        })
    }

    useEffect(() => {
        let user_id = parseInt(params.user_id);

        if (isNaN(user_id) || user_id === 0) {
            return;
        }

        fetchUser();
    }, [params.user_id]);

    return <>
        <Stack horizontal>
            <DefaultButton
                text="Save"
                primaryDisabled={sending}
                split
                iconProps={{iconName: "Save"}}
                onClick={sendUpdate}
                menuProps={{
                    items: [
                        {
                            key: "delete",
                            text: "Delete",
                            iconProps: {iconName: "Delete"},
                            onClick: () => setPrepDelete(true)
                        }
                    ]
                }}
            />
        </Stack>
        <Stack horizontal>
            <Persona size={PersonaSize.size120}/>
            <Stack horizontal tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <TextField placeholder="Full Name" value={user.full_name} onChange={(e, full_name) =>
                        setUser(v => ({...v, full_name}))
                    }/>
                    <TextField placeholder="Username" value={user.username} onChange={(e, username) => 
                        setUser(v => ({...v, username}))
                    }/>
                </Stack>
                <Stack tokens={{childrenGap: 8}}>
                    <Dropdown
                        styles={{"root": {width: 120}}}
                        options={[
                            {key: "admin", text: "Admin", selected: user.level === 1, data: 1},
                            {key: "manager", text: "Manager", selected: user.level === 10, data: 10},
                            {key: "user", text: "User", selected: user.level === 20, data: 20}
                        ]}
                        onChange={(e, o, i) => {
                            setUser(v => ({...v, level: o.data}));
                        }}
                    />
                </Stack>
            </Stack>
        </Stack>
        <IndentSection content="Personal Information">
            <Stack tokens={{childrenGap: 8}} styles={{root: {width: 250}}}>
                <TextField label="Email" value={user.email} onChange={(e, email) =>
                    setUser(v => ({...v, email}))
                }/>
            </Stack>
        </IndentSection>
        <IndentSection content="Authenticiation">
            <Stack tokens={{childrenGap: 8}} styles={{root: {width: 250}}}>
                <TextField label="Password" value={password} onChange={(e,v) => setPassword(v)}/>
            </Stack>
        </IndentSection>
        <Dialog
            hidden={!prep_delete}
            onDismiss={() => setPrepDelete(false)}
            dialogContentProps={{
                type: DialogType.normal,
                title: "Delete Field",
                subText: "Are you sure you want to delete this field?"
            }}
        >
            <DialogFooter>
                <DefaultButton
                    text="Yes"
                    primary
                    onClick={() => {
                        setPrepDelete(false);
                        sendDelete();
                    }}
                />
                <DefaultButton
                    text="No"
                    onClick={() => setPrepDelete(false)}
                />
            </DialogFooter>
        </Dialog>
    </>
}

const AdminUserIdView = () => {
    const history = useHistory();
    const location = useLocation();

    return <Stack 
        horizontal
        verticalAlign="center"
        horizontalAlign="center"
        style={{
            position: "absolute",
            top: 0, zIndex: 1,
            width: "100%",
            height: "100%",
            backgroundColor: "rgba(0,0,0,0.5)"
        }}
    >
        <Stack style={{
            width: 600, height: "100%",
            backgroundColor: "white",
            position: "relative"
        }}>
            <IconButton
                iconProps={{iconName: "Cancel"}}
                style={{position: "absolute", top: 0, right: 0, zIndex: 2}}
                onClick={() => {
                    let new_path = location.pathname.split("/");
                    new_path.pop();

                    history.push(new_path.join("/"));
                }}
            />
            <ScrollablePane>
                <Stack tokens={{childrenGap: 8, padding: 8}}>
                    <UserInformation/>
                </Stack>
            </ScrollablePane>
        </Stack>
    </Stack>
}

export default AdminUserIdView;
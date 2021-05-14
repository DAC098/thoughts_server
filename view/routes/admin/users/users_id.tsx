import { DefaultButton, Dialog, DialogFooter, DialogType, Dropdown, IconButton, Persona, PersonaSize, ScrollablePane, SearchBox, Separator, Stack, TextField } from "@fluentui/react"
import React, { Reducer, useEffect, useReducer, useState } from "react"
import { useHistory, useLocation, useParams } from "react-router"
import { cloneUserInfoJson, makeUserInfoJson, UserDataJson, UserInfoJson } from "../../../api/types"
import IndentSection from "../../../components/IndentSection"
import * as api from "../../../api"
import { json } from "../../../request"
import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import { SliceActionTypes } from "../../../redux/types"

interface AdminUserIdViewState {
    original: UserInfoJson
    current: UserInfoJson
    
    loading: boolean
    sending: boolean
    deleting: boolean
    loading_available_users: boolean
    
    available_users: UserDataJson[]
    known_users: {[key: string]: boolean}

    password: string

    prep_delete: boolean
    changes_made: boolean
}

function makeInitialState(): AdminUserIdViewState { 
    return {
        original: makeUserInfoJson(),
        current: makeUserInfoJson(),
    
        loading: false,
        sending: false,
        deleting: false,
        loading_available_users: false,
    
        available_users: [],
        known_users: {},
    
        password: "",
    
        prep_delete: false,
        changes_made: false
    }
}

const adminUserIdViewSlice = createSlice({
    name: "admin_user_id_view",
    initialState: makeInitialState(),
    reducers: {
        set_password: (state, action: PayloadAction<string>) => {
            state.password = action.payload;
        },

        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_sending: (state, action: PayloadAction<boolean>) => {
            state.sending = action.payload;
        },
        set_deleting: (state, action: PayloadAction<boolean>) => {
            state.deleting = action.payload;
        },
        set_loading_available_users: (state, action: PayloadAction<boolean>) => {
            state.loading_available_users = action.payload;
        },

        set_available_users: (state, action: PayloadAction<UserDataJson[]>) => {
            state.available_users = action.payload;
        },

        set_user: (state, action: PayloadAction<UserInfoJson>) => {
            state.original = action.payload;
            state.current = cloneUserInfoJson(action.payload);
            state.password = "";
            state.known_users = {};

            for (let user of state.current.user_access) {
                state.known_users[user.id] = true;
            }
        },
        update_user: (state, action: PayloadAction<any>) => {
            state.current[action.payload.key] = action.payload.value;
        },
        reset_user: (state) => {
            state.current = cloneUserInfoJson(state.original);
            state.password = "";
            state.known_users = {};

            for (let user of state.current.user_access) {
                state.known_users[user.id] = true;
            }
        },

        set_prep_delete: (state, action: PayloadAction<boolean>) => {
            state.prep_delete = action.payload;
        },

        add_user_access: (state, action: PayloadAction<UserDataJson>) => {
            if (state.known_users[action.payload.id] ?? false) {
                return;
            }

            state.current.user_access.push({
                id: action.payload.id,
                username: action.payload.username,
                full_name: action.payload.full_name,
                ability: "r"
            });
            state.known_users[action.payload.id] = true;
        },
        drop_user_access: (state, action: PayloadAction<number>) => {
            delete state.known_users[state.current.user_access[action.payload].id];
            state.current.user_access.splice(action.payload, 1);
        }
    }
});

const reducer_actions = {
    ...adminUserIdViewSlice.actions
};

type AdminUserIdViewActions = SliceActionTypes<typeof reducer_actions>;
type AdminUserIdViewReducer = Reducer<AdminUserIdViewState, AdminUserIdViewActions>;

const UserInformation = () => {
    const params = useParams<{user_id: string}>();
    const history = useHistory();

    let [state, dispatch] = useReducer<AdminUserIdViewReducer>(
        adminUserIdViewSlice.reducer, makeInitialState()
    );

    const sendUpdate = () => {
        if (state.sending) {
            return;
        }

        dispatch(reducer_actions.set_sending(true));

        let promise = null;
        
        if (params.user_id === "0") {
            promise = api.admin.users.post({
                ...state.current, 
                password: state.password
            }).then(u => {
                dispatch(reducer_actions.set_user(u));
                history.push(`/admin/users/${u.id}`);
            });
        } else {
            promise = api.admin.users.id.put(params.user_id, state.current).then(u => {
                dispatch(reducer_actions.set_user(u));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch(reducer_actions.set_sending(false));
        });
    }

    const sendDelete = () => {
        if (state.deleting) {
            return;
        }

        if (params.user_id === "0") {
            return;
        }

        dispatch(reducer_actions.set_deleting(true));

        json.delete(`/admin/users/${params.user_id}`).then(() => {
            history.push("/admin/users");
        }).catch((e) => {
            console.error(e);
            dispatch(reducer_actions.set_deleting(false));
        });
    }

    const fetchUser = async (): Promise<UserInfoJson | null> => {
        if (state.loading) {
            return;
        }

        dispatch(reducer_actions.set_loading(true));
        let u = null;
        
        try {
            u = await api.admin.users.id.get(params.user_id);
            dispatch(reducer_actions.set_user(u));
        } catch (e) {
            console.error(e);
        }

        dispatch(reducer_actions.set_loading(false));

        return u;
    }

    const fetchAvailableUsers = (current_level = state.current.level) => {
        if (state.loading_available_users || state.loading) {
            return;
        }

        dispatch(reducer_actions.set_loading_available_users(true));

        console.log(current_level);

        api.admin.users.get({level: current_level == 20 ? 10 : 20}).then(list => {
            dispatch(reducer_actions.set_available_users(list));
        }).catch(console.error).then(() => {
            dispatch(reducer_actions.set_loading_available_users(false));
        });
    }

    useEffect(() => {
        let user_id = parseInt(params.user_id);

        if (isNaN(user_id) || user_id === 0) {
            fetchAvailableUsers();
            return;
        }

        fetchUser().then(v => {
            if (v) fetchAvailableUsers(v.level)
        });
    }, [params.user_id]);

    let available_users_components = [];

    for (let user of state.available_users) {
        if (state.known_users[user.id] ?? false) {
            continue;
        }

        available_users_components.push(
            <Stack key={user.id} horizontal verticalAlign="center">
                <Stack.Item grow>
                    <Persona
                        text={user.full_name ?? user.username}
                        secondaryText={user.full_name != null ? user.username : null}
                    />
                </Stack.Item>
                <IconButton
                    iconProps={{iconName: "Add"}}
                    onClick={() => dispatch(reducer_actions.add_user_access(user))}
                />
            </Stack>
        );
    }

    return <>
        <Stack horizontal>
            <DefaultButton
                text="Save"
                primaryDisabled={state.sending}
                split
                iconProps={{iconName: "Save"}}
                onClick={sendUpdate}
                menuProps={{
                    items: [
                        {
                            key: "delete",
                            text: "Delete",
                            iconProps: {iconName: "Delete"},
                            onClick: () => dispatch(reducer_actions.set_prep_delete(true))
                        }
                    ]
                }}
            />
        </Stack>
        <Stack horizontal>
            <Persona size={PersonaSize.size120}/>
            <Stack horizontal tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <TextField placeholder="Full Name" value={state.current.full_name ?? ""} onChange={(e, full_name) =>
                        dispatch(reducer_actions.update_user({key: "full_name", value: full_name}))
                    }/>
                    <TextField placeholder="Username" value={state.current.username} onChange={(e, username) => 
                        dispatch(reducer_actions.update_user({key: "username", value: username}))
                    }/>
                </Stack>
                <Stack tokens={{childrenGap: 8}}>
                    <Dropdown
                        styles={{"root": {width: 120}}}
                        options={[
                            {key: "manager", text: "Manager", selected: state.current.level === 10, data: 10},
                            {key: "user", text: "User", selected: state.current.level === 20, data: 20}
                        ]}
                        onChange={(e, o, i) => {
                            dispatch(reducer_actions.update_user({key: "level", value: o.data}));
                            fetchAvailableUsers(o.data);
                        }}
                    />
                </Stack>
            </Stack>
        </Stack>
        <IndentSection content="Personal Information">
            <Stack tokens={{childrenGap: 8}} styles={{root: {width: 250}}}>
                <TextField label="Email" value={state.current.email ?? ""} onChange={(e, email) => 
                    dispatch(reducer_actions.update_user({key: "email", value: email}))
                }/>
            </Stack>
        </IndentSection>
        <IndentSection content="Authenticiation">
            <Stack tokens={{childrenGap: 8}} styles={{root: {width: 250}}}>
                <TextField label="Password" value={state.password} onChange={(e,v) => dispatch(reducer_actions.set_password(v))}/>
            </Stack>
        </IndentSection>
        <IndentSection content="User Access">
            <Stack horizontal tokens={{childrenGap: 8}}>
                <Stack.Item grow>
                    <SearchBox defaultValue=""/>
                </Stack.Item>
            </Stack>
            <Stack horizontal styles={{root: {width: "100%"}}}>
                <Stack.Item styles={{"root": {width: "50%"}}}>
                    <Stack tokens={{childrenGap: 8}}>
                        <Separator children={"Available"}/>
                        {available_users_components}
                    </Stack>
                </Stack.Item>
                <Stack.Item grow>
                    <Stack tokens={{childrenGap: 8}}>
                        <Separator children={"Assigned"}/>
                        {state.current.user_access.map((v, index) =>
                            <Stack key={v.id} horizontal verticalAlign="center">
                                <Stack.Item grow>
                                    <Persona
                                        text={v.full_name ?? v.username}
                                        secondaryText={v.full_name != null ? v.username : null}
                                    />
                                </Stack.Item>
                                <IconButton 
                                    iconProps={{iconName: "Delete"}}
                                    onClick={() => dispatch(reducer_actions.drop_user_access(index))}
                                />
                            </Stack>
                        )}
                    </Stack>
                </Stack.Item>
            </Stack>
        </IndentSection>
        <Dialog
            hidden={!state.prep_delete}
            onDismiss={() => dispatch(reducer_actions.set_prep_delete(false))}
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
                        dispatch(reducer_actions.set_prep_delete(false));
                        sendDelete();
                    }}
                />
                <DefaultButton
                    text="No"
                    onClick={() => dispatch(reducer_actions.set_prep_delete(false))}
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
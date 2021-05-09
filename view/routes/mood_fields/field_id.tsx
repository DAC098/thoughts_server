import { DefaultButton, Dialog, DialogFooter, DialogType, Dropdown, IconButton, IDropdownOption, Stack, TextField } from "@fluentui/react"
import React, { createContext, Dispatch, useContext, useEffect, useReducer } from "react"
import { useHistory, useLocation, useParams } from "react-router"
import { cloneMoodFieldJson, makeMoodFieldJson, MoodFieldJson } from "../../api/types"
import * as api from "../../api"
import { useOwner } from "../../hooks/useOwner"
import { makeMoodFieldType, MoodFieldType, MoodFieldTypeName } from "../../api/mood_field_types"
import { MoodFieldTypeEditView } from "../../components/mood_fields"
import { useAppDispatch } from "../../hooks/useApp"
import { actions as mood_fields_actions } from "../../redux/slices/mood_fields"

interface FieldState {
    original?: MoodFieldJson
    current?: MoodFieldJson
    loading: boolean
    sending: boolean
    changes_made: boolean
    prep_delete: boolean
    deleting: boolean

    edit_view: boolean
}

interface SetLoading {
    type: "set-loading"
    value: boolean
}

interface SetSending {
    type: "set-sending"
    value: boolean
}

interface PrepDelete {
    type: "prep-delete",
    value: boolean
}

interface SetDeleting {
    type: "set-deleting",
    value: boolean
}

interface SetEditView {
    type: "set-edit",
    value: boolean
}

interface SetField {
    type: "set-field",
    value: MoodFieldJson
}

interface ResetField {
    type: "reset-field"
}

interface NewField {
    type: "new-field"
}

interface ChangeConfigType {
    type: "change-config-type"
    value: string
}

interface UpdateConfig {
    type: "update-config"
    value: MoodFieldType
}

interface UpdateComment {
    type: "update-comment"
    value: string
}

interface UpdateName {
    type: "update-name"
    value: string
}

type FieldStateActions = SetLoading | SetSending |
    PrepDelete | SetDeleting |
    SetEditView |
    SetField | ResetField | NewField |
    ChangeConfigType | UpdateConfig |
    UpdateComment | UpdateName;

function fieldStateReducer(state: FieldState, action: FieldStateActions): FieldState {
    switch (action.type) {
        case "set-loading": {
            return {
                ...state,
                loading: action.value
            }
        }
        case "set-sending": {
            return {
                ...state,
                sending: action.value
            }
        }
        case "prep-delete": {
            return {
                ...state,
                prep_delete: action.value
            }
        }
        case "set-deleting": {
            return {
                ...state,
                deleting: action.value
            }
        }
        case "set-edit": {
            return {
                ...state,
                edit_view: action.value
            }
        }
        case "set-field": {
            return {
                ...state,
                changes_made: false,
                current: cloneMoodFieldJson(action.value),
                original: cloneMoodFieldJson(action.value)
            }
        }
        case "reset-field": {
            return {
                ...state,
                changes_made: false,
                current: cloneMoodFieldJson(state.original)
            }
        }
        case "new-field": {
            return {
                ...state,
                changes_made: false,
                current: makeMoodFieldJson(),
                original: makeMoodFieldJson()
            }
        }
        case "change-config-type": {
            let current = cloneMoodFieldJson(state.current);
            current.config = makeMoodFieldType((action.value as MoodFieldTypeName));

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "update-config": {
            let current = cloneMoodFieldJson(state.current);
            current.config = action.value;

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "update-comment": {
            let current = cloneMoodFieldJson(state.current);
            current.comment = action.value.length !== 0 ? action.value : null;

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "update-name": {
            let current = cloneMoodFieldJson(state.current);
            current.name = action.value;

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        default: {
            return {
                ...state
            }
        }
    }
}

const FieldStateContext = createContext<Dispatch<FieldStateActions>>(null);

interface FieldIdViewProps {
    user_specific?: boolean
}

const FieldIdView = ({user_specific = false}: FieldIdViewProps) => {
    const location = useLocation<{field?: MoodFieldJson}>();
    const history = useHistory();
    const params = useParams<{field_id: string, user_id?: string}>();
    const owner = useOwner(user_specific);
    const appDispatch = useAppDispatch();

    const allow_edit = params.user_id == null;

    const [state,dispatch] = useReducer(fieldStateReducer, {
        current: null, original: null,
        changes_made: false,
        loading: false,
        sending: false,
        prep_delete: false,
        deleting: false,
        edit_view: allow_edit && params.field_id === "0"
    })

    const fetchField = () => {
        if (state.loading) {
            return;
        }

        dispatch({type:"set-loading", value: true});

        (user_specific ?
            api.users.id.mood_fields.id.get(owner, params.field_id) :
            api.mood_fields.id.get(params.field_id)
        ).then((field) => {
            dispatch({type: "set-field", value: field});
        }).catch(console.error).then(() => {
            dispatch({type:"set-loading", value: false});
        });
    }

    const sendField = () => {
        if (user_specific) {
            return;
        }

        if (state.current == null) {
            return;
        }

        if (state.sending) {
            return;
        }

        dispatch({type: "set-sending", value: true});

        let promise = null;

        if (state.current.id) {
            promise = api.mood_fields.id.put(state.current.id, state.current).then(field => {
                history.replace(`/mood_fields/${field.id}`, {field});
                dispatch({type: "set-field", value: field});
                appDispatch(mood_fields_actions.update_field(field))
            });
        } else {
            promise = api.mood_fields.post(state.current).then(field => {
                history.push(`/mood_fields/${field.id}`, {field});
                dispatch({type: "set-field", value: field});
                appDispatch(mood_fields_actions.add_field(field));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch({type: "set-sending", value: false});
        })
    }

    const deleteField = () => {
        if (user_specific) {
            return;
        }

        if (state.current == null || state.current.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch({type: "set-deleting", value: true});

        api.mood_fields.id.del(state.current.id).then(() => {
            appDispatch(mood_fields_actions.delete_field(state.current.id));
            history.push("/mood_fields");
        }).catch(console.error);
    }

    useEffect(() => {
        let field_id = parseInt(params.field_id);

        if (isNaN(field_id) || field_id === 0) {
            dispatch({type: "new-field"});
            return;
        }
        
        fetchField();
    }, [params.field_id]);

    let options: IDropdownOption[] = [];

    for (let key in MoodFieldTypeName) {
        options.push({
            key,
            text: key,
            selected: state.current?.config.type === key ?? false
        });
    }

    return <FieldStateContext.Provider value={dispatch}>
        <Stack
            horizontal
            verticalAlign="center"
            horizontalAlign="center"
            style={{
                width: "100%", height: "100%",
                backgroundColor: "rgba(0,0,0,0.5)",
                position: "absolute",
                top: 0,
                zIndex: 1
            }}
        >
            <Stack style={{
                width: 600, height: "100%",
                backgroundColor: "white",
                position: "relative"
            }} tokens={{childrenGap: 8, padding: 8}}>
                {state.current != null ? 
                    <>
                        <Stack horizontal verticalAlign="end" tokens={{childrenGap: 8}}>
                            <TextField
                                placeholder="Name"
                                value={state.current.name}
                                disabled={!state.edit_view}
                                onChange={(e,v) => dispatch({type: "update-name", value: v})}
                            />
                            <Dropdown
                                style={{minWidth: 130}}
                                options={options}
                                disabled={!state.edit_view}
                                onChange={(e, o, i) => {
                                    dispatch({type: "change-config-type", value: (o.key as string)})
                                }}
                            />
                            {allow_edit ?
                                <IconButton 
                                    iconProps={{iconName: "Edit"}} 
                                    onClick={() => dispatch({type: "set-edit", value: !state.edit_view})}
                                />
                                :
                                null
                            }
                            {state.edit_view ?
                                <DefaultButton
                                    text="Save"
                                    primaryDisabled={!state.changes_made}
                                    split
                                    iconProps={{iconName: "Save"}}
                                    onClick={sendField}
                                    menuProps={{
                                        items: [
                                            {
                                                key: "reset",
                                                text: "Reset",
                                                disabled:  !state.changes_made,
                                                iconProps: {iconName: "Refresh"},
                                                onClick: () => dispatch({type: "reset-field"})
                                            },
                                            {
                                                key: "delete",
                                                text: "Delete",
                                                iconProps: {iconName: "Delete"},
                                                onClick: () => dispatch({type: "prep-delete", value: true})
                                            }
                                        ]
                                    }}
                                />
                                :
                                null
                            }
                        </Stack>
                        <MoodFieldTypeEditView config={state.current.config} onChange={conf => dispatch({type: "update-config", value: conf})}/>
                        <TextField
                            label="Comment"
                            value={state.current.comment ?? ""}
                            onChange={(e,v) => dispatch({type: "update-comment", value: v})}
                        />
                    </>
                    :
                    state.loading ?
                        <h4>Loading</h4>
                        :
                        <h4>No Field to show</h4>
                }
                <IconButton 
                    iconProps={{iconName: "Cancel"}}
                    style={{position: "absolute", top: 0, right: 0}}
                    onClick={() => {
                        let new_path = location.pathname.split("/");
                        new_path.pop();

                        history.push(new_path.join("/"));
                    }}
                />
            </Stack>
        </Stack>
        <Dialog
            hidden={!state.prep_delete}
            onDismiss={() => dispatch({type: "prep-delete", value: false})}
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
                        dispatch({type: "prep-delete", value: false});
                        deleteField();
                    }}
                />
                <DefaultButton
                    text="No"
                    onClick={() => dispatch({type: "prep-delete", value: false})}
                />
            </DialogFooter>
        </Dialog>
    </FieldStateContext.Provider>
}

export default FieldIdView;
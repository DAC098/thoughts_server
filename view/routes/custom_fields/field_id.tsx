import { DefaultButton, Dialog, DialogFooter, DialogType, Dropdown, IconButton, IDropdownOption, Position, SpinButton, Stack, TextField } from "@fluentui/react"
import React, { createContext, Dispatch, Reducer, useContext, useEffect, useReducer } from "react"
import { useHistory, useLocation, useParams } from "react-router"
import { cloneCustomFieldJson, makeCustomField, CustomField } from "../../api/types"
import api from "../../api"
import { useOwner } from "../../hooks/useOwner"
import { makeCustomFieldType, CustomFieldType, CustomFieldTypeName } from "../../api/custom_field_types"
import { CustomFieldTypeEditView } from "../../components/custom_fields"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { custom_field_actions } from "../../redux/slices/custom_fields"
import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import { SliceActionTypes } from "../../redux/types"

interface FieldState {
    original?: CustomField
    current?: CustomField
    loading: boolean
    sending: boolean
    changes_made: boolean
    prep_delete: boolean
    deleting: boolean

    edit_view: boolean
}

function makeInitialState(edit_view: boolean): FieldState {
    return {
        current: null, original: null,
        changes_made: false,
        loading: false,
        sending: false,
        prep_delete: false,
        deleting: false,
        edit_view
    }
}

const fieldStateSlice = createSlice({
    name: "field_state",
    initialState: makeInitialState(false),
    reducers: {
        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_sending: (state, action: PayloadAction<boolean>) => {
            state.sending = action.payload;
        },
        set_prep_delete: (state, action: PayloadAction<boolean>) => {
            state.prep_delete = action.payload;
        },
        set_deleting: (state, action: PayloadAction<boolean>) => {
            state.deleting = action.payload;
        },
        set_edit_view: (state, action: PayloadAction<boolean>) => {
            state.edit_view = action.payload;
        },

        set_field: (state, action: PayloadAction<CustomField>) => {
            state.original = action.payload;
            state.current = cloneCustomFieldJson(action.payload);
            state.changes_made = false;
        },
        reset_field: (state) => {
            state.current = cloneCustomFieldJson(state.original);
            state.changes_made = false;
        },
        new_field: (state) => {
            state.original = makeCustomField();
            state.current = makeCustomField();
            state.changes_made = false;
        },

        change_config_type: (state, action: PayloadAction<CustomFieldTypeName>) => {
            state.current.config = makeCustomFieldType(action.payload);
            state.changes_made = true;
        },
        update_config: (state, action: PayloadAction<CustomFieldType>) => {
            state.current.config = action.payload;
            state.changes_made = true;
        },

        update_comment: (state,action: PayloadAction<string>) => {
            state.current.comment = action.payload.length !== 0 ? action.payload : null;
            state.changes_made = true;
        },
        update_name: (state,action: PayloadAction<string>) => {
            state.current.name = action.payload;
            state.changes_made = true;
        },
        update_order: (state, action: PayloadAction<number>) => {
            state.current.order = action.payload;
            state.changes_made = true;
        }
    }
});

const reducer_actions = {
    ...fieldStateSlice.actions
}

type FieldStateActionsTypes = SliceActionTypes<typeof reducer_actions>;
type FieldStateReducer = Reducer<FieldState, FieldStateActionsTypes>;

interface FieldIdViewProps {
    user_specific?: boolean
}

const FieldIdView = ({user_specific = false}: FieldIdViewProps) => {
    const location = useLocation<{field?: CustomField}>();
    const history = useHistory();
    const params = useParams<{field_id: string, user_id?: string}>();
    const owner = useOwner(user_specific);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const appDispatch = useAppDispatch();

    const allow_edit = params.user_id == null;

    const [state,dispatch] = useReducer<FieldStateReducer>(
        fieldStateSlice.reducer, 
        makeInitialState(allow_edit && params.field_id === "0")
    );

    const fetchField = () => {
        if (state.loading) {
            return;
        }

        dispatch(reducer_actions.set_loading(true));

        (user_specific ?
            api.users.id.custom_fields.id.get(owner, params.field_id) :
            api.custom_fields.id.get(params.field_id)
        ).then((field) => {
            dispatch(reducer_actions.set_field(field));
        }).catch(console.error).then(() => {
            dispatch(reducer_actions.set_loading(false));
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

        dispatch(reducer_actions.set_sending(true));

        let promise = null;

        if (state.current.id) {
            promise = api.custom_fields.id.put(state.current.id, state.current).then(field => {
                history.replace(`/custom_fields/${field.id}`);
                dispatch(reducer_actions.set_field(field));
                appDispatch(custom_field_actions.update_field(field))
            });
        } else {
            promise = api.custom_fields.post(state.current).then(field => {
                history.push(`/custom_fields/${field.id}`);
                dispatch(reducer_actions.set_field(field));
                appDispatch(custom_field_actions.add_field(field));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch(reducer_actions.set_sending(false));
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

        dispatch(reducer_actions.set_deleting(true));

        api.custom_fields.id.del(state.current.id).then(() => {
            appDispatch(custom_field_actions.delete_field(state.current.id));
            history.push("/custom_fields");
        }).catch((e) => {
            console.error(e);
            dispatch(reducer_actions.set_deleting(false));
        });
    }

    useEffect(() => {
        let field_id = parseInt(params.field_id);

        if (isNaN(field_id) || field_id === 0) {
            dispatch(reducer_actions.new_field());
            return;
        }
        
        if (custom_fields_state.loading) {
            fetchField();
        } else {
            for (let field of custom_fields_state.custom_fields) {
                if (field.id === field_id) {
                    dispatch(reducer_actions.set_field(cloneCustomFieldJson(field)));
                    return;
                }
            }

            fetchField();
        }
    }, [params.field_id]);

    let options: IDropdownOption[] = [];

    for (let key in CustomFieldTypeName) {
        options.push({
            key,
            text: key,
            selected: state.current?.config.type === key ?? false
        });
    }

    return <>
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
                                onChange={(e,v) => dispatch(reducer_actions.update_name(v))}
                            />
                            <Dropdown
                                style={{minWidth: 130}}
                                options={options}
                                disabled={!state.edit_view}
                                onChange={(e, o, i) => {
                                    dispatch(reducer_actions.change_config_type(o.key as CustomFieldTypeName))
                                }}
                            />
                            {allow_edit ?
                                <IconButton 
                                    iconProps={{iconName: "Edit"}} 
                                    onClick={() => dispatch(reducer_actions.set_edit_view(!state.edit_view))}
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
                                                onClick: () => dispatch(reducer_actions.reset_field())
                                            },
                                            {
                                                key: "delete",
                                                text: "Delete",
                                                iconProps: {iconName: "Delete"},
                                                onClick: () => dispatch(reducer_actions.set_prep_delete(true))
                                            }
                                        ]
                                    }}
                                />
                                :
                                null
                            }
                        </Stack>
                        <CustomFieldTypeEditView 
                            config={state.current.config} 
                            onChange={conf => dispatch(reducer_actions.update_config(conf))}
                        />
                        <SpinButton
                            label="Order"
                            labelPosition={Position.top}
                            value={state.current.order.toString()}
                            styles={{root: {width: 200}}}
                            onChange={(e,v) => {
                                let int = parseInt(v);

                                if (!isNaN(int)) {
                                    dispatch(reducer_actions.update_order(int));
                                }
                            }}
                        />
                        <TextField
                            label="Comment"
                            value={state.current.comment ?? ""}
                            onChange={(e,v) => dispatch(reducer_actions.update_comment(v))}
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
                        deleteField();
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

export default FieldIdView;
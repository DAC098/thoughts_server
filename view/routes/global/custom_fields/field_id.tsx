import { DefaultButton, Dialog, DialogFooter, DialogType, Dropdown, IconButton, IDropdownOption, Stack, TextField } from "@fluentui/react";
import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import React from "react";
import { Reducer, useEffect, useReducer } from "react";
import { useHistory, useLocation, useParams } from "react-router";
import apiv2 from "../../../apiv2";
import { CustomFieldTypeName, makeCustomFieldType, CustomFieldType } from "../../../apiv2/custom_field_types";
import { CustomField, cloneCustomField, newCustomField, GlobalCustomField, cloneGlobalCustomField, newGlobalCustomField } from "../../../apiv2/types";
import { CustomFieldTypeEditView } from "../../../components/custom_fields";
import OverlayedPage from "../../../components/OverlayedPage";
import { SliceActionTypes } from "../../../redux/types";
import { stringFromLocation, urlFromLocation } from "../../../util/url";

interface FieldState {
    original?: GlobalCustomField
    current?: GlobalCustomField
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

        set_field: (state, action: PayloadAction<GlobalCustomField>) => {
            state.original = action.payload;
            state.current = cloneGlobalCustomField(action.payload);
            state.changes_made = false;
        },
        reset_field: (state) => {
            state.current = cloneGlobalCustomField(state.original);
            state.changes_made = false;
        },
        new_field: (state) => {
            state.original = newGlobalCustomField(CustomFieldTypeName.Integer);
            state.current = newGlobalCustomField(CustomFieldTypeName.Integer);
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
        }
    }
});

const reducer_actions = {
    ...fieldStateSlice.actions
}

type FieldStateActionsTypes = SliceActionTypes<typeof reducer_actions>;
type FieldStateReducer = Reducer<FieldState, FieldStateActionsTypes>;

interface GlobalCustomFieldIdViewProps {}

const GlobalCustomFieldIdView = ({}: GlobalCustomFieldIdViewProps) => {
    const location = useLocation();
    const history = useHistory();
    const params = useParams<{field_id: string}>();
    const [state, dispatch] = useReducer<FieldStateReducer>(
        fieldStateSlice.reducer,
        makeInitialState(params.field_id === "0")
    );

    const fetchGlobalField = () => {
        if (state.loading) {
            return;
        }

        dispatch(reducer_actions.set_loading(true));

        apiv2.global.custom_fields.id.get({
            id: params.field_id
        }).then((res) => {
            dispatch(reducer_actions.set_field(res.body.data))
        }).catch(console.error).then(() => {
            dispatch(reducer_actions.set_loading(false));
        });
    }

    const sendField = () => {
        if (state.current == null) {
            return;
        }

        if (state.sending) {
            return;
        }

        dispatch(reducer_actions.set_sending(true));

        let promise = null;

        if (state.current.id) {
            promise = apiv2.global.custom_fields.id.put({
                id: state.current.id,
                post: state.current
            }).then((res) => {
                let field = res.body.data;
                dispatch(reducer_actions.set_field(field))
            })
        } else {
            promise = apiv2.global.custom_fields.post({
                post: state.current
            }).then(res => {
                let field = res.body.data;
                history.replace(stringFromLocation({
                    ...location,
                    pathname: `/global/custom_fields/${field.id}`
                }));
                dispatch(reducer_actions.set_field(field));
            })
        }

        promise.catch(console.error).then(() => {
            dispatch(reducer_actions.set_sending(false))
        });
    }

    const deleteField = () => {
        if (state.current == null || state.current.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch(reducer_actions.set_deleting(true));

        apiv2.global.custom_fields.id.del({id: state.current.id}).then(() => {
            let url = urlFromLocation(location);

            if (url.searchParams.has("prev")) {
                history.push(url.searchParams.get("prev"))
            } else {
                let new_path = location.pathname.split("/");
                new_path.pop();

                history.push(stringFromLocation({
                    ...location,
                    pathname: new_path.join("/")
                }))
            }
        })
    }

    useEffect(() => {
        if (params.field_id === "0") {
            dispatch(reducer_actions.new_field());
        } else {
            fetchGlobalField();
        }
    }, []);

    let options: IDropdownOption[] = [];

    for (let key in CustomFieldTypeName) {
        options.push({
            key,
            text: key,
            selected: state.current?.config.type === key ?? false
        })
    }

    return <>
        <OverlayedPage>
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
                                dispatch(reducer_actions.change_config_type(o.key as CustomFieldTypeName));
                            }}
                        />
                        <IconButton
                            iconProps={{iconName: "Edit"}}
                            onClick={() => dispatch(reducer_actions.set_edit_view(!state.edit_view))}
                        />
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
                                            disabled: !state.changes_made,
                                            iconProps: {iconName: "Refresh"},
                                            onClick: () => dispatch(reducer_actions.reset_field())
                                        },
                                        {
                                            key: "delete",
                                            text: "Delete",
                                            disabled: state.current.id === 0,
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
                    let url = urlFromLocation(location);

                    if (url.searchParams.has("prev")) {
                        history.push(url.searchParams.get("prev"))
                    } else {
                        let new_path = location.pathname.split("/");
                        new_path.pop();

                        history.push(stringFromLocation({
                            ...location,
                            pathname: new_path.join("/")
                        }))
                    }
                }}
            />
        </OverlayedPage>
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

export default GlobalCustomFieldIdView;
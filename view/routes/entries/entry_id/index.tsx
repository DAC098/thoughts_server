import { DatePicker, DefaultButton, Dialog, DialogFooter, DialogType, IconButton, Label, Separator, Stack, Text, TextField } from "@fluentui/react"
import React, { useContext, useEffect, useReducer } from "react"
import { useHistory, useLocation, useParams } from "react-router-dom"
import * as api from "../../../api"
import { MoodEntryType } from "../../../api/mood_entry_types"
import { EntryJson, MoodEntryJson, MoodFieldJson, TextEntryJson } from "../../../api/types"
import { MoodEntryTypeEditView, MoodEntryTypeReadView } from "../../../components/mood_entries"
import { useAppDispatch, useAppSelector } from "../../../hooks/useApp"
import { EntryStateContext, entryStateReducer } from "./reducer"
import { actions as entries_actions } from "../../../redux/entries"

interface TextEntryEditViewProps {
    text_entries: TextEntryJson[]
}

const TextEntryEditView = ({text_entries}: TextEntryEditViewProps) => {
    let dispatch = useContext(EntryStateContext);

    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v, index) => {
            return <Stack key={v.id} horizontal tokens={{childrenGap: 8}}>
                <Stack.Item grow>
                    <TextField key={v.id} multiline autoAdjustHeight value={v.thought} onChange={(e,t) => {
                        dispatch({type: "update-text-entry", index, thought: t});
                    }}/>
                </Stack.Item>
                <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                    dispatch({type: "delete-text-entry", index});
                }}/>
            </Stack>
        })}
    </Stack>
}

interface TextEntryReadViewProps {
    text_entries: TextEntryJson[]
}

const TextEntryReadView = ({text_entries}: TextEntryReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v) => {
            let line_splits = v.thought.split(/\n/);
            let total = line_splits.length;

            return <div key={v.id}>
                {line_splits.map((t, i) =>
                    <div key={i} style={{paddingBottom: total - 1 !== i ? 4 : 0}}>
                        <Text>{t}</Text>
                    </div>
                )}
            </div>
        })}
    </Stack>
}

interface MoodEntryInputProps {
    field: MoodFieldJson
    entry: MoodEntryJson

    onDelete?: () => void
    onChange?: (entry: {comment: string, value: MoodEntryType}) => void
}

const MoodEntryInputs = ({
    field,
    entry,
    onDelete,
    onChange
}: MoodEntryInputProps) => {
    let similar_types = entry.value.type === field.config.type;

    return <Stack tokens={{childrenGap: 8}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <Label title={similar_types ? field.comment : ""}>{field.name}</Label>
            <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                onDelete?.();
            }}/>
        </Stack>
        <MoodEntryTypeEditView value={entry.value} config={similar_types ? field.config : null} onChange={value => {
            onChange?.({comment: entry.comment, value});
        }}/>
        <TextField type="text" placeholder="comment" value={entry.comment ?? ""} onChange={(e,v) => {
            onChange?.({comment: v, value: entry.value});
        }}/>
    </Stack>
}

interface MoodEntriesEditViewProps {
    mood_fields: {[id: string]: MoodFieldJson}
    mood_entries: MoodEntryJson[]
}

const MoodEntriesEditView = ({mood_fields, mood_entries}: MoodEntriesEditViewProps) => {
    let dispatch = useContext(EntryStateContext);

    return <Stack tokens={{childrenGap: 8}}>
        {mood_entries.map((mood_entry,index) =>
            <MoodEntryInputs
                key={mood_entry.id}
                field={mood_fields?.[mood_entry.field_id]}
                entry={mood_entry}
        
                onDelete={() => dispatch({type: "delete-mood-entry", index})}
                onChange={(value) => dispatch({type:"update-mood-entry", index, value})}
            />
        )}
    </Stack>
}

interface MoodEntriessReadViewProps {
    mood_entries: MoodEntryJson[]
    mood_fields: {[id: string]: MoodFieldJson}
}

const MoodEntriesReadView = ({mood_fields, mood_entries}: MoodEntriessReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {mood_entries.map((mood_entry, index) => 
            <Stack key={mood_entry.id} tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <Separator alignContent="start">{mood_entry.field}</Separator>
                    <MoodEntryTypeReadView 
                        value={mood_entry.value} 
                        config={mood_fields?.[mood_entry.field_id]?.config}
                    />
                </Stack>
                <Text>{mood_entry.comment}</Text>
            </Stack>
        )}
    </Stack>
}

interface EntryIdProps {
    user_specific?: boolean
}

const EntryId = ({user_specific = false}: EntryIdProps) => {
    const width = 600;
    const location = useLocation<{entry?: EntryJson}>();
    const history = useHistory();
    const params = useParams<{entry_id: string, user_id?: string}>();
    
    const entries_state = useAppSelector(state => state.entries);
    const mood_fields_state = useAppSelector(state => state.mood_fields);
    const appDispatch = useAppDispatch();

    const allow_edit = params.user_id == null;

    let [state, dispatch] = useReducer(entryStateReducer, {
        current: null, original: null,
        loading: false, sending: false,
        existing_fields: {},
        changes_made: false,
        prep_delete: false,
        deleting: false,
        edit_view: allow_edit && params.entry_id === "0",
        invalid: false
    });

    const fetchEntry = () => {
        if (state.loading) {
            return;
        }

        dispatch({type: "set-loading", value: true});

        (user_specific ?
            api.users.id.entries.id.get(params.user_id, params.entry_id) :
            api.entries.id.get(params.entry_id)
        ).then(entry => {
            dispatch({type: "set-entry", entry});
        }).catch(console.error).then(() => {
            dispatch({type: "set-loading", value: false});
        })
    }

    const sendEntry = () => {
        if (user_specific)
            return;

        if (state.current == null)
            return;
        
        if (state.sending)
            return;

        dispatch({type:"set-sending", value: true});

        let promise = null;
        
        if (state.current.id) {
            promise = api.entries.id.put(state.current.id, {
                created: state.current.created,
                mood_entries: state.current.mood_entries.map(v => {
                    return {
                        id: v.id,
                        field_id: v.field_id,
                        value: v.value,
                        comment: v.comment
                    }
                }),
                text_entries: state.current.text_entries.map(v => {
                    return {id: v.id, thought: v.thought}
                })
            }).then(entry => {
                history.replace(`/entries/${entry.id}`, {entry});
                dispatch({type: "set-entry", entry});
                appDispatch(entries_actions.update_entry(entry));
            })
        } else {
            promise = api.entries.post({
                created: state.current.created,
                mood_entries: state.current.mood_entries.map(v => {
                    return {
                        field_id: v.field_id,
                        value: v.value,
                        comment: v.comment
                    }
                }),
                text_entries: state.current.text_entries.map(v => {
                    return {thought: v.thought}
                })
            }).then(entry => {
                history.push(`/entries/${entry.id}`, {entry});
                dispatch({type: "set-entry", entry});
                appDispatch(entries_actions.add_entry(entry));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch({type: "set-sending", value: false});
        });
    }

    const deleteEntry = () => {
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

        api.entries.id.del(state.current.id).then(() => {
            appDispatch(entries_actions.delete_entry(state.current.id));
            history.push("/entries");
        }).catch(console.error);
    }

    useEffect(() => {
        let entry_id = parseInt(params.entry_id);

        if (isNaN(entry_id) || entry_id === 0) {
            dispatch({type: "new-entry"});
            return;
        }
        
        if (entries_state.loading) {
            fetchEntry();
        } else {
            for (let entry of entries_state.entries) {
                if (entry.id === entry_id) {
                    dispatch({type:"set-entry", entry});
                    return;
                }
            }

            fetchEntry();
        }
    }, [params.entry_id]);

    useEffect(() => {
        dispatch({type: "set-mood-fields", fields: mood_fields_state.mood_fields});
    }, [mood_fields_state.mood_fields]);

    let mood_field_options = [];

    for (let field_id in (mood_fields_state.mapping ?? {})) {
        if (field_id in state.existing_fields) {
            continue;
        }

        mood_field_options.push({
            key: mood_fields_state.mapping[field_id].name,
            text: mood_fields_state.mapping[field_id].name,
            title: mood_fields_state.mapping[field_id].comment,
            onClick: () => {
                dispatch({type: "create-mood-entry-action", field: field_id})
            }
        });
    }

    return <EntryStateContext.Provider value={dispatch}>
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
            <Stack 
                style={{
                    width: width, height: "100%",
                    backgroundColor: "white",
                    position: "relative"
                }}
            >
                <Stack.Item grow={0} shrink={0} style={{
                    position:"sticky", 
                    top: 0, zIndex: 2, 
                    backgroundColor: "white",
                    paddingTop: 8,
                    paddingBottom: 8,
                    paddingLeft: 8
                }}>
                    {state.current != null ? 
                        <Stack horizontal tokens={{childrenGap: 8}}>
                            <DatePicker
                                disabled={!state.edit_view}
                                value={new Date(state.current.created)}
                                onSelectDate={d => {
                                    dispatch({type: "update-entry", created: d.toISOString()})
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
                                <>
                                    <DefaultButton
                                        text="Add"
                                        iconProps={{iconName: "Add"}}
                                        menuProps={{items: [
                                            {key: "new-text-entry", text: "Text Entry", onClick: () => {
                                                dispatch({type: "create-text-entry-action"});
                                            }},
                                            {
                                                key: "new-mood-entry",
                                                text: "Mood Entry",
                                                disabled: mood_field_options.length === 0,
                                                subMenuProps: {
                                                    items: mood_field_options
                                                }
                                            }
                                        ]}}
                                    />
                                    <DefaultButton
                                        text="Save"
                                        primaryDisabled={!state.changes_made}
                                        split
                                        iconProps={{iconName: "Save"}}
                                        onClick={() => sendEntry()}
                                        menuProps={{
                                            items: [
                                                {
                                                    key: "reset",
                                                    text: "Reset",
                                                    disabled: !state.changes_made,
                                                    iconProps: {iconName: "Refresh"},
                                                    onClick: () => dispatch({type: "reset-entry"})
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
                                </>
                                :
                                null
                            }
                        </Stack>
                        :
                        state.loading ?
                            <h4>Loading</h4>
                            :
                            <h4>No Entry to Show</h4>
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
                </Stack.Item>
                <Stack style={{overflowY: "auto"}} tokens={{childrenGap: 8, padding: 8}}>
                {state.current != null ?
                    state.edit_view ?
                        <>
                            <TextEntryEditView text_entries={state.current.text_entries}/>
                            <div style={{width: width * (2/3)}}>
                                {!mood_fields_state.loading ?
                                    <MoodEntriesEditView 
                                        mood_fields={mood_fields_state.mapping} 
                                        mood_entries={state.current.mood_entries}
                                    />
                                    :
                                    <h6>Loading</h6>
                                }
                            </div>
                        </>
                        :
                        <>
                            <TextEntryReadView text_entries={state.current.text_entries}/>
                            <div style={{width: width * (2/3)}}>
                                {!mood_fields_state.loading ?
                                    <MoodEntriesReadView 
                                        mood_fields={mood_fields_state.mapping}
                                        mood_entries={state.current.mood_entries}
                                    />
                                    :
                                    <h6>Loading</h6>
                                }
                            </div>
                        </>
                    :
                    null
                }
                </Stack>
            </Stack>
        </Stack>
        <Dialog
            hidden={!state.prep_delete}
            onDismiss={() => dispatch({type: "prep-delete", value: false})}
            dialogContentProps={{
                type: DialogType.normal,
                title: "Delete Entry",
                subText: "Are you sure you want to delete this entry?"
            }}
        >
            <DialogFooter>
                <DefaultButton
                    text="Yes"
                    primary
                    onClick={() => {
                        dispatch({type: "prep-delete", value: false});
                        deleteEntry();
                    }}
                />
                <DefaultButton
                    text="No"
                    onClick={() => dispatch({type: "prep-delete", value: false})}
                />
            </DialogFooter>
        </Dialog>
    </EntryStateContext.Provider>
}

export default EntryId;
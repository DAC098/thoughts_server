import { DatePicker, DefaultButton, IconButton, Label, SpinButton, Stack, TextField } from "@fluentui/react";
import React, { createContext, Dispatch, useContext, useEffect, useReducer } from "react"
import { useHistory, useLocation, useParams } from "react-router";
import { cloneEntryJson, EntryJson, getEntry, getMoodFields, makeEntryJson, makeMoodEntryJson, makeTextEntry, MoodEntryJson, MoodFieldJson, TextEntryJson } from "../../json";
import { json } from "../../request";
import { getCreatedDateToString, getCreatedStringToDate } from "../../time";

interface TextEntryUI extends TextEntryJson {
    key?: string
}

interface EntryUIState extends EntryJson {
    text_entries: TextEntryUI[]
}

interface EntryState {
    original?: EntryUIState
    current?: EntryUIState
    loading: boolean
    sending: boolean
    fields: {[id: string]: MoodFieldJson}
    existing_fields: {[id: string]: number}
    loading_fields: boolean
    changes_made: boolean
}

interface CreateMoodEntryAction {
    type: "create-mood-entry-action",
    field: string
}

interface UpdateMoodEntryAction {
    type: "update-mood-entry"
    index: number
    low: number
    high?: number
    comment?: string
}

interface DeleteMoodEntryAction {
    type: "delete-mood-entry"
    index: number
}

interface CreateTextEntryAction {
    type: "create-text-entry-action"
}

interface UpdateTextEntryAction {
    type: "update-text-entry"
    index: number
    thought: string
}

interface DeleteTextEntryAction {
    type: "delete-text-entry"
    index: number
}

interface UpdateEntryAction {
    type: "update-entry"
    created: string
}

interface SetEntry {
    type: "set-entry"
    entry: EntryJson
}

interface SetLoading {
    type: "set-loading"
    value: boolean
}

interface SetSending {
    type: "set-sending"
    value: boolean
}

interface SetMoodFields {
    type: "set-mood-fields"
    fields: MoodFieldJson[]
}

interface SetLoadingFields {
    type: "set-loading-fields"
    value: boolean
}

interface ResetEntry {
    type: "reset-entry"
}

interface NewEntry {
    type: "new-entry"
}

type EntryStateActions = UpdateMoodEntryAction | UpdateTextEntryAction | UpdateEntryAction |
    SetEntry | SetLoading | SetSending |
    ResetEntry | NewEntry |
    SetMoodFields | SetLoadingFields |
    CreateMoodEntryAction | CreateTextEntryAction |
    DeleteMoodEntryAction | DeleteTextEntryAction;

function entryStateReducer(state: EntryState, action: EntryStateActions): EntryState {
    switch (action.type) {
        case "set-entry":{
            let original = action.entry;
            let current = cloneEntryJson(action.entry);
            let existing_fields = {};

            for (let field of current.mood_entries) {
                existing_fields[field.field_id] = field.id;
            }

            return {
                ...state,
                original,
                current,
                existing_fields,
                changes_made: false
            };
        }
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
        case "reset-entry": {
            let current = cloneEntryJson(state.original);
            let existing_fields = {};

            for (let field of current.mood_entries) {
                existing_fields[field.field_id] = field.id;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: false
            }
        }
        case "new-entry": {
            let original = makeEntryJson();
            original.created = getCreatedDateToString(new Date());
            let current = makeEntryJson();
            current.created = original.created.slice(0);

            return {
                ...state,
                original,
                current,
                existing_fields: {},
                changes_made: true
            }
        }
        case "update-entry": {
            let current = cloneEntryJson(state.current);
            current.created = action.created;

            return {
                ...state,
                current,
                changes_made: true
            };
        }
        case "create-mood-entry-action": {
            let field = state.fields[action.field];

            if (field == null) {
                console.log("field requested was not found");
                return {
                    ...state
                };
            }

            if (field.id in state.existing_fields) {
                console.log("field requested already exists");
                return {
                    ...state
                };
            }

            let current = cloneEntryJson(state.current);
            let existing_fields = {};
            let mood_entry = makeMoodEntryJson();
            mood_entry.field = field.name;
            mood_entry.field_id = field.id;
            mood_entry.is_range = field.is_range;
            
            if (field.is_range) {
                mood_entry.high = 0;
            }

            current.mood_entries.push(mood_entry);

            for (let f of current.mood_entries) {
                existing_fields[f.field_id] = 0;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: true
            }
        }
        case "update-mood-entry": {
            let current = cloneEntryJson(state.current);
            current.mood_entries[action.index].low = action.low;
            current.mood_entries[action.index].high = action.high;
            current.mood_entries[action.index].comment = action.comment;

            return {
                ...state,
                current,
                changes_made: true
            };
        }
        case "delete-mood-entry": {
            let existing_fields = {};
            let current = cloneEntryJson(state.current);
            current.mood_entries.splice(action.index, 1);

            for (let f of current.mood_entries) {
                existing_fields[f.field_id] = f.id;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: true
            }
        }
        case "create-text-entry-action": {
            let current = cloneEntryJson(state.current);
            let text_entry: TextEntryUI = makeTextEntry();
            text_entry.key = Date.now().toString();
            current.text_entries.push(text_entry);

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "update-text-entry": {
            let current = cloneEntryJson(state.current);
            current.text_entries[action.index].thought = action.thought;

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "delete-text-entry": {
            let current = cloneEntryJson(state.current);
            current.text_entries.splice(action.index, 1);

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "set-mood-fields": {
            let mapping = {};

            for (let field of action.fields) {
                mapping[field.id] = field;
            }

            return {
                ...state,
                fields: mapping
            }
        }
        default: {
            return {
                ...state
            }
        }
    }
}

const EntryStateContext = createContext<Dispatch<EntryStateActions>>(null);

interface TextEntryAreaProps {
    text_entries: TextEntryJson[]
}

const TextEntryArea = ({text_entries}: TextEntryAreaProps) => {
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

interface MoodEntryInputProps {
    field: string
    low: number
    high: number
    comment: string

    is_range: boolean

    min?: number
    max?: number

    onLow?: (v: number) => void
    onHigh?: (v: number) => void
    onComment?: (v: string) => void
    onDelete?: () => void
}

const MoodEntryInputs = ({
    field,
    low, high, comment,
    is_range,
    min, max,
    onLow, onHigh, onComment,
    onDelete
}: MoodEntryInputProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <Label>{field}</Label>
            <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                onDelete?.();
            }}/>
        </Stack>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                label="Low"
                value={low.toString()}
                min={min} max={max}
                onChange={(e,v) => {
                    let int = parseInt(v);
                    onLow?.(isNaN(int) ? 0 : int);
                }}
            />
            <SpinButton
                label="High"
                disabled={!is_range}
                value={high.toString()}
                min={min} max={max}
                onChange={(e,v) => {
                    let int = parseInt(v);
                    onHigh?.(isNaN(int) ? 0 : int)
                }}
            />
        </Stack>
        <TextField type="text" placeholder="comment" value={comment} onChange={(e,v) => {
            onComment?.(v);
        }}/>
    </Stack>
}

interface MoodEntriesAreaProps {
    mood_fields: {[id: string]: MoodFieldJson}
    mood_entries: MoodEntryJson[]
}

const MoodEntriesArea = ({mood_fields, mood_entries}: MoodEntriesAreaProps) => {
    let dispatch = useContext(EntryStateContext);

    return <Stack tokens={{childrenGap: 8}}>
        {mood_entries.map((mood_entry,index) => 
            <MoodEntryInputs
                key={mood_entry.id}
                field={mood_entry.field}
                low={mood_entry.low} high={mood_entry.high ?? 0} comment={mood_entry.comment ?? ""}
                min={mood_fields?.[mood_entry.field_id].minimum ?? null}
                max={mood_fields?.[mood_entry.field_id].maximum ?? null}
                is_range={mood_entry.is_range}
        
                onLow={v => dispatch({type: "update-mood-entry", index, low: v, high: mood_entry.high, comment: mood_entry.comment })}
                onHigh={v => dispatch({type: "update-mood-entry", index, low: mood_entry.low, high: v, comment: mood_entry.comment })}
                onComment={v => dispatch({type: "update-mood-entry", index, low: mood_entry.low, high: mood_entry.high, comment: v })}
                onDelete={() => dispatch({type: "delete-mood-entry", index})}
            />
        )}
    </Stack>
}

interface CreatedDateFieldProps {
    created: string
}

const CreatedDateField = ({created}: CreatedDateFieldProps) => {
    let dispatch = useContext(EntryStateContext);
    
    return <DatePicker 
        value={getCreatedStringToDate(created)}
        onSelectDate={d => {
            dispatch({type: "update-entry", created: getCreatedDateToString(d)})
        }}
        formatDate={getCreatedDateToString}
    />
}

const EntryId = () => {
    const width = 600;
    let location = useLocation();
    let params = useParams<{entry_id: string}>();
    let history = useHistory();

    let [state, dispatch] = useReducer(entryStateReducer, {
        current: null, original: null,
        loading: false, sending: false,
        fields: null, loading_fields: false,
        existing_fields: {},
        changes_made: false
    });

    const fetchEntry = (id: number) => {
        if (state.loading) {
            return;
        }

        dispatch({type: "set-loading", value: true});

        getEntry(id).then(entry => {
            dispatch({type: "set-entry", entry});
            history.replace(location.pathname, {entry});
        }).catch(err => {
            console.log(err);
        }).then(() => {
            dispatch({type: "set-loading", value: false});
        })
    }

    const sendEntry = () => {
        if (state.current == null)
            return;
        
        if (state.sending)
            return;

        dispatch({type:"set-sending", value: true});

        let path = "/entries";
        let is_post = true;

        if (state.current.id) {
            path += "/" + state.current.id;
            is_post = false;
        }
        
        json[is_post ? "post" : "put"]<EntryJson>(path, {
            created: state.current.created,
            mood_entries: state.current.mood_entries,
            text_entries: state.current.text_entries.map(v => {
                return {id: v.id, thought: v.thought}
            })
        }).then(({body}) => {
            if (is_post) {
                history.push(`/entries/${body.data.id}`);
            } else {
                dispatch({type: "set-entry", entry: body.data});
            }
        }).catch(err => {
            console.error(err);
        }).then(() => {
            dispatch({type: "set-sending", value: false});
        });
    }

    useEffect(() => {
        let entry_id = parseInt(params.entry_id);

        if (!isNaN(entry_id) && entry_id !== 0) {
            fetchEntry(entry_id)
        } else {
            dispatch({type: "new-entry"});
        }
    }, [params.entry_id]);

    useEffect(() => {
        dispatch({type: "set-loading-fields", value: true});
        getMoodFields().then(list => {
            dispatch({type: "set-mood-fields", fields: list});
        }).catch(err => {
            console.error(err);
        }).then(() => {
            dispatch({type: "set-loading-fields", value: false});
        })
    }, []);

    let mood_field_options = [];

    for (let field_id in (state.fields ?? {})) {
        if (field_id in state.existing_fields) {
            continue;
        }

        mood_field_options.push({
            key: state.fields[field_id].name,
            text: state.fields[field_id].name,
            title: state.fields[field_id].comment,
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
                    width: `${width}px`, height: "100%",
                    backgroundColor: "white",
                    position: "relative"
                }}
                onClick={(e) => {
                    e.stopPropagation();
                }}
            >
                <Stack.Item grow={0} shrink={0} style={{
                    position:"sticky", 
                    top: 0, zIndex: 2, 
                    backgroundColor: "rgba(0,0,0,0.5)",
                    paddingTop: 8,
                    paddingBottom: 8
                }}>
                    {state.current != null ? 
                        <Stack horizontal tokens={{childrenGap: 8}}>
                            <CreatedDateField created={state.current.created}/>
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
                                disabled={!state.changes_made}
                                onClick={() => sendEntry()}
                            />
                            <DefaultButton
                                text="Reset"
                                disabled={!state.changes_made}
                                onClick={() => dispatch({type: "reset-entry"})}
                            />
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
                            history.push("/entries");
                        }}
                    />
                </Stack.Item>
                {state.current != null ?
                    <Stack style={{overflowY: "auto"}} tokens={{childrenGap: 8, padding: 8}}>
                        <TextEntryArea text_entries={state.current.text_entries}/>
                        <div style={{width: `${width * (2/3)}px`}}>
                            <MoodEntriesArea mood_fields={state.fields} mood_entries={state.current.mood_entries}/>
                        </div>
                    </Stack>
                    :
                    null
                }
            </Stack>
        </Stack>
    </EntryStateContext.Provider>
}

export default EntryId;
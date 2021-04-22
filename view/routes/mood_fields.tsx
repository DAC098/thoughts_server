import { DefaultButton, IconButton, Label, SpinButton, Stack, TextField, Toggle } from "@fluentui/react";
import React, { useEffect, useReducer, useState } from "react"
import { useHistory } from "react-router";
import useSending from "../hooks/useSending";
import { cloneMoodFieldJson, getMoodFields, makeMoodFieldJson, MoodFieldJson } from "../json";
import { json } from "../request";

interface MoodFieldInputProps {
    name: string
    is_range: boolean
    comment: string

    button_text: string
    button_disabled: boolean

    view_second?: boolean
    second_button_text?: string
    second_button_disabled?: boolean

    enable_min?: boolean
    min?: number

    enable_max?: boolean
    max?: number

    onName?: (v: string) => void
    onIsRange?: (v: boolean) => void
    onComment?: (v: string) => void

    onButton?: () => void
    onSecondButton?: () => void

    onToggleMin?: (v: boolean) => void
    onToggleMax?: (v: boolean) => void

    onMin?: (v: number) => void
    onMax?: (v: number) => void
}

const MoodFieldInput = ({
    name, is_range, comment,
    button_text, button_disabled,
    view_second, second_button_text, second_button_disabled,
    enable_min, enable_max,
    min, max,
    onName, onIsRange, onComment,
    onButton, onSecondButton,
    onToggleMin, onToggleMax,
    onMin, onMax
}: MoodFieldInputProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <TextField label="Name" value={name} onChange={(e,v) => {
                onName?.(v);
            }}/>
            <Toggle label="Is Range" checked={is_range} onChange={(e,v) => {
                onIsRange?.(!is_range);
            }}/>
        </Stack>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <Stack tokens={{childrenGap: 8}}>
                <Label>Minimum</Label>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Toggle checked={enable_min} onChange={(e,v) => {
                        onToggleMin?.(!enable_min)
                    }}/>
                    <SpinButton 
                        disabled={!enable_min}
                        value={(min ?? 0).toString()}
                        onChange={(e,v) => {
                            let int = parseInt(v);
                            onMin?.(isNaN(int) ? 0 : int)
                        }}
                    />
                </Stack>
            </Stack>
            <Stack tokens={{childrenGap: 8}} style={{width: "50%"}}>
                <Label>Maximum</Label>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Toggle checked={enable_max} onChange={(e,v) => {
                        onToggleMax?.(!enable_max)
                    }}/>
                    <SpinButton
                        disabled={!enable_max}
                        value={(max ?? 0).toString()}
                        onChange={(e, v) => {
                            let int = parseInt(v);
                            onMax?.(isNaN(int) ? 0 : int)
                        }}
                    />
                </Stack>
            </Stack>
        </Stack>
        <TextField label="Comment" value={comment} onChange={(e,v) => {
            onComment?.(v);
        }}/>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <DefaultButton text={button_text} disabled={button_disabled} onClick={() => {
                onButton?.();
            }}/>
            {(view_second ?? false) ?
                <DefaultButton text={second_button_text} disabled={second_button_disabled} onClick={() => {
                    onSecondButton?.();
                }}/>
                :
                null
            }
        </Stack>
    </Stack>
}

interface NewMoodFieldProps {
    onCreated?: () => void
}

const NewMoodField = ({onCreated}: NewMoodFieldProps) => {
    let [name, setName] = useState("");

    let [enable_min, setEnableMin] = useState(false);
    let [min, setMin] = useState(0);

    let [enable_max, setEnableMax] = useState(false);
    let [max, setMax] = useState(0);

    let [is_range, setIsRange] = useState(false);
    let [comment, setComment] = useState("");

    let [sending, sendMoodField] = useSending(() => {
        return json.post('/mood_fields', {
            name, is_range,
            minimum: enable_min ? min : null,
            maximum: enable_max ? max : null,
            comment: comment.length ? comment : null
        })
            .then(({}) => {
                onCreated?.();
                setName("");
                setIsRange(false);
                setComment("");
                setEnableMin(false);
                setMin(0);
                setEnableMax(false);
                setMax(0);
            })
            .catch(err => {
                console.error(err);
            })
    });

    return <MoodFieldInput
        name={name} is_range={is_range} comment={comment}
        onName={setName} onIsRange={setIsRange} onComment={setComment}

        enable_min={enable_min} enable_max={enable_max} 
        min={min} max={max}
        onToggleMin={v => {
            setEnableMin(v);
            setMin(0);
        }}
        onMin={v => setMin(v)}
        onToggleMax={v => {
            setEnableMax(v);
            setMax(0);
        }}
        onMax={v => setMax(v)}
        
        button_text={"Create Field"} button_disabled={
            sending || name.length === 0
        }
        onButton={sendMoodField}
    />
}

interface SetMoodField {
    type: "set-mood-field",
    value: MoodFieldJson
}

interface SetName {
    type: "set-name"
    value: string
}

interface SetIsRange {
    type: "set-is-range"
    value: boolean
}

interface SetEnableMin {
    type: "set-enable-min"
    value: boolean
}

interface SetEnableMax {
    type: "set-enable-max"
    value: boolean
}

interface SetMin {
    type: "set-min"
    value: number
}

interface SetMax {
    type: "set-max"
    value: number
}

interface SetComment {
    type: "set-comment"
    value: string
}

type CurrentMoodFieldActions = SetMoodField |
    SetName |
    SetIsRange |
    SetEnableMin | SetEnableMax |
    SetMin | SetMax |
    SetComment;

interface MoodFieldState extends MoodFieldJson {
    enable_min: boolean
    enable_max: boolean
    changed: boolean
}

function currentMoodFieldReducer(state: MoodFieldState, action: CurrentMoodFieldActions) {
    switch (action.type) {
        case "set-mood-field": {
            let field = cloneMoodFieldJson(action.value);

            return {
                enable_min: field.minimum != null,
                enable_max: field.maximum != null,
                changed: false,
                ...field
            };
        }
        case "set-name": return {
            ...state,
            name: action.value,
            changed: true
        }
        case "set-enable-min": return {
            ...state,
            enable_min: action.value,
            minimum: 0,
            changed: true
        }
        case "set-enable-max": return {
            ...state,
            enable_max: action.value,
            maximum: 0,
            changed: true
        }
        case "set-min": return {
            ...state,
            minimum: action.value,
            changed: true
        }
        case "set-max": return {
            ...state,
            maximum: action.value,
            changed: true
        }
        case "set-is-range": return {
            ...state,
            is_range: action.value,
            changed: true
        }
        case "set-comment": return {
            ...state,
            comment: action.value,
            changed: true
        }
        default: return {
            ...state
        }
    }
}

interface CurrentMoodFieldProps {
    mood_field: MoodFieldJson

    onUpdated?: () => void
}

const CurrentMoodField = ({mood_field, onUpdated}: CurrentMoodFieldProps) => {
    let [state, dispatch] = useReducer(currentMoodFieldReducer, {
        ...cloneMoodFieldJson(mood_field),
        enable_min: mood_field.minimum != null,
        enable_max: mood_field.maximum != null,
        changed: false
    });
    let [updating, updateMoodField] = useSending(() => {
        return json.put(`/mood_fields/${mood_field.id}`, {
            name: state.name,
            is_range: state.is_range,
            minimum: state.enable_min ? state.minimum : null,
            maximum: state.enable_max ? state.maximum : null,
            comment: state.comment != null && state.comment.length ? state.comment : null
        })
            .then(({}) => {
                onUpdated?.();
            })
            .catch(err => {
                console.error(err);
            })
    });
    let [deleting, deleteMoodField] = useSending(() => {
        return json.delete(`/mood_fields/${mood_field.id}`)
            .then(({}) => {
                onUpdated?.();
            })
            .catch(err => {
                console.error(err)
            })
    });

    useEffect(() => {
        dispatch({type: "set-mood-field", value: mood_field});
    }, [mood_field]);

    return <MoodFieldInput
        name={state.name} is_range={state.is_range} comment={state.comment}
        onName={v =>dispatch({type: "set-name", value: v})} 
        onIsRange={v => dispatch({type: "set-is-range", value: v})} 
        onComment={v => dispatch({type: "set-comment", value: v})}

        enable_min={state.enable_min} enable_max={state.enable_max} 
        min={state.minimum} max={state.maximum}
        onToggleMin={v => dispatch({type: "set-enable-min", value: v})}
        onMin={v => dispatch({type: "set-min", value: v})}
        onToggleMax={v => dispatch({type: "set-enable-max", value: v})}
        onMax={v => dispatch({type: "set-max", value: v})}

        button_text={"Update"} 
        button_disabled={updating || deleting || state.name.length === 0 || !state.changed}

        onButton={updateMoodField}

        view_second second_button_text="Delete" second_button_disabled={updating || deleting}
        onSecondButton={deleteMoodField}
    />
}

const MoodFieldsView = () => {
    let history = useHistory();

    let [fields, setFields] = useState<MoodFieldJson[]>([]);
    let [loading, getFields] = useSending(() => {
        return getMoodFields()
            .then(fields => {
                setFields(fields);
            })
            .catch(err => {
                console.error(err);
            })
    })

    useEffect(() => {
        getFields();
    }, [])

    return <Stack
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
            tokens={{padding: 12, childrenGap: 8}}
            style={{
                width: "auto", height: "100%",
                position: "relative",
                backgroundColor: "white",
                overflowY: "auto"
            }}
        >
            <IconButton
                iconProps={{iconName: "Cancel"}} 
                style={{position: "absolute", top: 0, right: 0}}
                onClick={() => {
                    history.push("/entries");
                }}
            />
            <NewMoodField onCreated={getFields}/>
            <Stack tokens={{childrenGap: 8}}>
                {fields.map(v => 
                    <CurrentMoodField key={v.id} mood_field={v} onUpdated={getFields}/>
                )}
            </Stack>
        </Stack>
    </Stack>
}

export default MoodFieldsView;
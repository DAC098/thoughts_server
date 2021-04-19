import { DefaultButton, Stack, TextField, Toggle } from "@fluentui/react";
import React, { useEffect, useState } from "react"
import useSending from "../hooks/useSending";
import { getMoodFields, MoodFieldJson } from "../json";
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

    onName?: (v: string) => void
    onIsRange?: (v: boolean) => void
    onComment?: (v: string) => void

    onButton?: () => void
    onSecondButton?: () => void
}

const MoodFieldInput = ({
    name, is_range, comment,
    button_text, button_disabled,
    view_second, second_button_text, second_button_disabled,
    onName, onIsRange, onComment,
    onButton, onSecondButton
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
        <div>
            <TextField label="Comment" value={comment} onChange={(e,v) => {
                onComment?.(v);
            }}/>
        </div>
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
    let [is_range, setIsRange] = useState(false);
    let [comment, setComment] = useState("");
    let [sending, sendMoodField] = useSending(() => {
        return json.post('/mood_fields', {
            name, is_range, comment: comment.length ? comment : null
        })
            .then(({}) => {
                onCreated?.();
                setName("");
                setIsRange(false);
                setComment("");
            })
            .catch(err => {
                console.error(err);
            })
    });

    return <MoodFieldInput
        name={name} is_range={is_range} comment={comment}
        onName={setName} onIsRange={setIsRange} onComment={setComment}
        button_text={"Create Field"} button_disabled={
            sending || name.length === 0
        }
        onButton={sendMoodField}
    />
}

interface CurrentMoodFieldProps {
    mood_field: MoodFieldJson

    onUpdated?: () => void
}

const CurrentMoodField = ({mood_field, onUpdated}: CurrentMoodFieldProps) => {
    let [name, setName] = useState(mood_field.name);
    let [is_range, setIsRange] = useState(mood_field.is_range);
    let [comment, setComment] = useState(mood_field.comment ?? "");
    let [updating, updateMoodField] = useSending(() => {
        return json.put(`/mood_fields/${mood_field.id}`, {
            name, is_range, comment: comment.length ? comment : null
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
        setName(mood_field.name);
        setIsRange(mood_field.is_range);
        setComment(mood_field.comment ?? "");
    }, [mood_field]);

    return <MoodFieldInput
        name={name} is_range={is_range} comment={comment}
        onName={setName} onIsRange={setIsRange} onComment={setComment}
        button_text={"Update"} button_disabled={
            updating || deleting || name.length === 0 || (mood_field.name === name && mood_field.is_range === is_range && (
                mood_field.comment == null ? comment.length === 0 : mood_field.comment === comment
            ))
        }
        onButton={updateMoodField}

        view_second second_button_text="Delete" second_button_disabled={updating || deleting}
        onSecondButton={deleteMoodField}
    />
}

const MoodFieldsView = () => {
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

    return <>
        <Stack tokens={{padding: 12, childrenGap: 8}}>
            <NewMoodField onCreated={getFields}/>
            <Stack tokens={{childrenGap: 8}}>
                {fields.map(v => 
                    <CurrentMoodField key={v.id} mood_field={v} onUpdated={getFields}/>
                )}
            </Stack>
        </Stack>
    </>
}

export default MoodFieldsView;
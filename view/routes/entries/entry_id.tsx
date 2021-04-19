import { DatePicker, DefaultButton, IconButton, Label, SpinButton, Stack, TextField } from "@fluentui/react";
import React, { useEffect, useState } from "react"
import { useHistory, useLocation, useParams } from "react-router";
import { EntryJson, getEntry, getMoodFields, MoodEntryJson, MoodFieldJson, TextEntryJson } from "../../json";
import { json } from "../../request";
import { compareDates, getCreatedDateToString, getCreatedStringToDate } from "../../time";

interface NewTextEntryProps {
    entry_id: number

    onCreated?: () => void
}

const NewTextEntry = ({entry_id, onCreated}: NewTextEntryProps) => {
    let [text, setText] = useState("");
    let [sending, setSending] = useState(false);

    const sendTextEntry = () => {
        if (sending)
            return;
        
        setSending(true);

        json.post(`/entries/${entry_id}/text_entries`, [{thought: text}])
            .then(body => {
                onCreated?.();
                setText("");
            })
            .catch(err => {
                console.error(err);
            })
            .then(() => {
                setSending(false)
            })
    }

    return <Stack tokens={{childrenGap: 8}}>
        <TextField multiline autoAdjustHeight value={text} onChange={(e,v) => {
            setText(v);
        }}/>
        <DefaultButton text="Save Text" primary disabled={sending || text.length === 0} onClick={() => {
            sendTextEntry();
        }}/>
    </Stack>
}

interface CurrentTextEntryProps {
    entry_id: number
    text_entry: TextEntryJson

    onUpdated?: () => void
}

const CurrentTextEntry = ({entry_id, text_entry, onUpdated}: CurrentTextEntryProps) => {
    let [current, setCurrent] = useState(text_entry.thought);
    let [text, setText] = useState(text_entry.thought);
    let [sending, setSending] = useState(false);

    const sendUpdate = () => {
        if (sending) {
            return;
        }

        setSending(true);

        json.put(`/entries/${entry_id}/text_entries/${text_entry.id}`, {thought: text})
            .then(({}) => {
                onUpdated?.();
            })
            .catch(err => {
                console.error(err);
            })
            .then(() => {
                setSending(false)
            });
    }

    const sendDelete = () => {
        if (sending) {
            return;
        }

        setSending(true);

        json.delete(`/entries/${entry_id}/text_entries/${text_entry.id}`)
            .then(() => {
                onUpdated?.();
            })
            .catch(err => {
                console.error(err);
            })
            .then(() => {
                setSending(false)
            });
    }

    useEffect(() => {
        setCurrent(text_entry.thought);
        setText(text_entry.thought);
    }, [text_entry.thought])

    return <Stack tokens={{childrenGap: 8}}>
        <TextField multiline autoAdjustHeight value={text} onChange={(e,v) => {
            setText(v);
        }}/>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <DefaultButton text="Update" disabled={text === current || sending} onClick={() => {
                sendUpdate();
            }}/>
            <DefaultButton text="Delete" disabled={sending} onClick={() => {
                sendDelete();
            }}/>
        </Stack>
    </Stack>
}

interface TextEntryAreaProps {
    entry_id: number
    text_entries: TextEntryJson[]

    onCreated?: () => void
    onUpdated?: () => void
}

const TextEntryArea = ({entry_id, text_entries, onCreated, onUpdated}: TextEntryAreaProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        <NewTextEntry entry_id={entry_id} onCreated={onCreated}/>
        {text_entries.map(v => {
            return <CurrentTextEntry key={v.id} entry_id={entry_id} text_entry={v} onUpdated={onUpdated}/>
        })}
    </Stack>
}

interface MoodEntryInputProps {
    field: string
    low: number
    high: number
    comment: string

    is_range: boolean

    disable_save: boolean
    save_text: string

    onLow?: (v: number) => void
    onHigh?: (v: number) => void
    onComment?: (v: string) => void

    onSave?: () => void
}

const MoodEntryInputs = ({
    field,
    low, high, comment,
    is_range,
    disable_save, save_text,
    onLow, onHigh, onComment, onSave
}: MoodEntryInputProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        <div>
            <Label>{field}</Label>
        </div>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                label="Low"
                value={low.toString()} 
                onIncrement={(v,e) => {
                    onLow?.(low + 1);
                }}
                onDecrement={(_v,e) => {
                    onLow?.(low - 1);
                }}
                onValidate={(v,e) => {
                    onLow?.(parseInt(v));
                }}
            />
            <SpinButton
                label="High"
                disabled={!is_range}
                value={high.toString()}
                onIncrement={(v,e) => {
                    onHigh?.(high + 1);
                }}
                onDecrement={(v,e) => {
                    onHigh?.(high - 1);
                }}
                onValidate={(v,e) => {
                    onHigh?.(parseInt(v));
                }}
            />
        </Stack>
        <TextField type="text" placeholder="comment" value={comment} onChange={(e,v) => {
            onComment?.(v);
        }}/>
        <div>
            <DefaultButton
                text={save_text} 
                disabled={disable_save}
                onClick={() => {
                    onSave?.();
                }}
            />
        </div>
    </Stack>
}

interface NewMoodEntryProps {
    entry_id: number
    mood_field: MoodFieldJson

    onCreated?: () => void
}

const NewMoodEntry = ({entry_id, mood_field, onCreated}: NewMoodEntryProps) => {
    let [low, setLow] = useState(0);
    let [high, setHigh] = useState(0);
    let [comment, setComment] = useState("");
    let [sending, setSending] = useState(false);

    const sendMoodEntry = () => {
        if (sending)
            return;
        
        setSending(true);

        json.post(`/entries/${entry_id}/mood_entries`, [{
            field_id: mood_field.id,
            low: low,
            high: mood_field.is_range ? high : null,
            comment: comment.length > 0 ? comment : null
        }])
            .then(({}) => {
                onCreated?.();
            })
            .catch((err) => {
                console.error(err);
            })
            .then(() => {
                setSending(false)
            });
    }

    return <MoodEntryInputs
        field={mood_field.name}
        low={low} high={high} comment={comment}
        is_range={mood_field.is_range}
        save_text="Save"
        disable_save={false}

        onLow={v => setLow(v)}
        onHigh={v => setHigh(v)}
        onComment={v => setComment(v)}
        onSave={() => sendMoodEntry()}
    />
}

interface CurrentMoodEntryProps {
    entry_id: number
    mood_entry: MoodEntryJson

    onUpdated?: () => void
}

const CurrentMoodEntry = ({entry_id, mood_entry, onUpdated}: CurrentMoodEntryProps) => {
    let [low, setLow] = useState(mood_entry.low);
    let [high, setHigh] = useState((mood_entry.high ?? 0));
    let [comment, setComment] = useState(mood_entry.comment);
    let [sending, setSending] = useState(false);

    const updateMoodEntry = () => {
        if (sending)
            return;

        setSending(true);

        json.put(`/entries/${entry_id}/mood_entries/${mood_entry.id}`, {
            low: low,
            high: mood_entry.is_range ? high : null,
            comment: comment?.length ? comment : null
        })
            .then(({}) => {
                onUpdated?.();
            })
            .catch(err => {
                console.error(err);
            })
            .then(() => {
                setSending(false)
            });
    }

    return <MoodEntryInputs
        field={mood_entry.field}
        low={low} high={high} comment={comment}
        is_range={mood_entry.is_range}
        save_text="Update"
        disable_save={low === mood_entry.low && high === mood_entry.high && comment === mood_entry.comment}

        onLow={v => setLow(v)}
        onHigh={v => setHigh(v)}
        onComment={v => setComment(v)}
        onSave={() => updateMoodEntry()}
    />
}


interface MoodEntriesAreaProps {
    entry_id: number
    mood_entries: MoodEntryJson[]

    onCreated?: () => void
    onUpdated?: () => void
}

const MoodEntriesArea = ({entry_id, mood_entries, onCreated, onUpdated}: MoodEntriesAreaProps) => {
    let [fields, setFields] = useState<{[id: number]: MoodFieldJson}>();
    let rendered_fields: number[] = [];

    useEffect(() => {
        getMoodFields()
            .then(list => {
                setFields(() => {
                    let rtn = {};

                    for (let f of list) {
                        rtn[f.id] = f;
                    }

                    return rtn;
                });
            })
            .catch(err => {
                console.error(err);
            })
    }, []);

    let given_fields = [];
    let missing_fields = [];

    for (let mf of mood_entries) {
        rendered_fields.push(mf.field_id);

        given_fields.push(<CurrentMoodEntry key={mf.id} entry_id={entry_id} mood_entry={mf} onUpdated={onUpdated}/>);
    }

    for (let f_id in fields) {
        if (parseInt(f_id) === rendered_fields[0]) {
            rendered_fields.shift();
            continue;
        }

        missing_fields.push(<NewMoodEntry key={f_id} entry_id={entry_id} mood_field={fields[f_id]} onCreated={onCreated}/>);
    }

    return <Stack tokens={{childrenGap: 8}}>
        {given_fields}
        {missing_fields}
    </Stack>
}

interface CreatedDateFieldProps {
    entry_id: number
    created: string

    onUpdated?: () => void
}

const CreatedDateField = ({entry_id, created, onUpdated}: CreatedDateFieldProps) => {
    let [sending, setSending] = useState(false);
    let [current, setCurrent] = useState(getCreatedStringToDate(created));
    let [date, setDate] = useState(getCreatedStringToDate(created));

    const sendUpdate = (d: Date) => {
        if (sending) {
            return;
        }

        setSending(true);

        json.put(`/entries/${entry_id}`, {created: getCreatedDateToString(d)})
            .then(({body}) => {
                onUpdated?.();
            })
            .catch(err => {
                console.log(err)
            })
            .then(() => {
                setSending(false)
            });
    }
    
    useEffect(() => {
        setCurrent(getCreatedStringToDate(created));
        setDate(getCreatedStringToDate(created));
    }, [created]);
    
    return <Stack horizontal tokens={{childrenGap: 8, padding: "12px 0 0"}}>
        <DatePicker 
            value={date}
            onSelectDate={d => {
                setDate(d)
            }}
            formatDate={getCreatedDateToString}
        />
        <DefaultButton
            text="Save Date"
            disabled={compareDates(current, date)}
            onClick={() => {
                sendUpdate(date);
            }}
        />
    </Stack>
}

const EntryId = () => {
    const width = 600;
    let location = useLocation();
    let params = useParams<{entry_id: string}>();
    let history = useHistory();

    let [loading, setLoading] = useState(false);
    let [entry, setEntry] = useState<EntryJson>(null);

    const fetchEntry = (id: number) => {
        if (loading) {
            return;
        }

        setLoading(true);

        getEntry(id).then(entry => {
            setEntry(entry);
            history.replace(location.pathname, {entry});
        }).catch(err => {
            console.log(err);
        }).then(() => {
            setLoading(false);
        })
    }

    useEffect(() => {
        if (entry === null) {
            let entry_id = parseInt(params.entry_id);

            if (entry_id != null) {
                fetchEntry(entry_id)
            } else {
                console.log("failed to get entry id from path");
            }
        }

    }, [entry]);

    return <Stack 
        horizontal 
        verticalAlign="center" 
        horizontalAlign="center" 
        style={{
            width: "100vw",height: "100vh",
            backgroundColor: "rgba(0,0,0,0.5)",
            position: "absolute",
            top: 0
        }}
        onClick={() => {
            history.push("/entries");
        }}
    >
        <Stack 
            style={{
                width: `${width}px`, height: "100vh",
                backgroundColor: "white",
                position: "relative",
                overflowY: "auto"
            }}
            tokens={{
                padding: "0 12px",
                childrenGap: 8
            }}
            onClick={(e) => {
                e.stopPropagation();
            }}
        >
            <Stack.Item grow={0} shrink={0} style={{position:"sticky", top: 0, zIndex: 1, backgroundColor: "white"}}>
                {entry != null ? 
                    <CreatedDateField entry_id={entry.id} created={entry.created} onUpdated={() => {
                        fetchEntry(entry.id);
                    }}/>
                    :
                    loading ?
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
            {entry != null ?
                <>
                    <TextEntryArea 
                        entry_id={entry.id} text_entries={entry.text_entries}
                        onCreated={() => fetchEntry(entry.id)}
                        onUpdated={() => fetchEntry(entry.id)}
                    />
                    <div style={{width: `${width * (2/3)}px`}}>
                        <MoodEntriesArea
                            entry_id={entry.id} mood_entries={entry.mood_entries}
                            onCreated={() => fetchEntry(entry.id)}
                            onUpdated={() => fetchEntry(entry.id)}
                        />
                    </div>
                </>
                :
                null
            }
        </Stack>
    </Stack>
}

export default EntryId;
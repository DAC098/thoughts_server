import { DatePicker, DefaultButton, IconButton, Stack } from "@fluentui/react"
import React, { useEffect, useState } from "react"
import { Link, Route, useLocation } from "react-router-dom"
import { json } from "../request"
import EntryId from "./entries/entry_id"
import {EntryJson, getEntries} from "../json"
import { getCreatedDateToString } from "../time"

interface NewEntrySectionProps {
    onCreated?: () => void
}

const NewEntrySection = ({onCreated}: NewEntrySectionProps) => {
    let [created, setCreated] = useState<Date>(new Date());
    let [sending, setSending] = useState(false);

    const sendEntry = (created: Date) => {
        if (sending) {
            return;
        }

        setSending(true);
        
        let month = created.getMonth() + 1;
        let day = created.getDate();

        json.post("/entries", {created: `${created.getFullYear()}-${month < 10 ? `0${month}` : month}-${day < 10 ? `0${day}` : day}`})
            .then(({}) => {
                setCreated(new Date());
                onCreated?.();
            }).catch(err => {
                if (err.type === "EntryExists") {
                }
            }).then(() => {
                setSending(false);
            });
    }
    
    return <form
        onSubmit={e => {
            e.preventDefault();

            sendEntry(created);
        }}
    >
        <Stack horizontal tokens={{
            childrenGap: 8
        }}>
            <DatePicker
                placeholder="Entry Date"
                value={created}
                onSelectDate={d => {
                    setCreated(d)
                }}
                formatDate={getCreatedDateToString}
            />
            <Stack.Item>
                <DefaultButton 
                    text="Create Entry"
                    primary
                    onClick={() => {
                        sendEntry(created);
                    }}
                    primaryDisabled={sending}
                />
            </Stack.Item>
        </Stack>
    </form>
}

interface EntryListItemProps {
    entry: EntryJson

    onDelete?: () => void
}

const EntryListItem = ({entry, onDelete}: EntryListItemProps) => {
    let [sending, setSending] = useState(false);

    const sendDelete = () => {
        if (sending)
            return;

        setSending(true);

        json.delete(`/entries/${entry.id}`)
            .then(({}) => {
                onDelete?.();
            })
            .catch(err => {
                console.error(err);
            })
            .then(() => {
                setSending(false);
            })
    }
    
    return <Stack horizontal verticalAlign="center" tokens={{
        childrenGap: 8
    }}>
        <span>{entry.created}</span>
        <Stack horizontal>
            <Link 
                to={{
                    pathname: `/entries/${entry.id}`,
                    state: {entry}
                }}
            >
                <IconButton 
                    title="Edit"
                    iconProps={{iconName: "Edit"}}
                />
            </Link>
            <IconButton
                title="Delete"
                iconProps={{iconName: "Delete"}}
                onClick={() => sendDelete()}
            />
        </Stack>
    </Stack>
}

const Entries = () => {
    let [entries, setEntries] = useState<EntryJson[]>([]);
    let [loading, setLoading] = useState(false);

    const location = useLocation();

    const loadEntries = () => {
        if (loading) {
            return;
        }

        setLoading(true);

        getEntries().then(entries => {
            setEntries(entries);
        }).catch(console.log).then(() => {
            setLoading(false);
        });
    }

    useEffect(() => {
        loadEntries();
    },[location.pathname]);

    return <>
        <Stack tokens={{padding: 12, childrenGap: 8}}>
            <NewEntrySection onCreated={() => loadEntries()}/>
            <Stack tokens={{childrenGap: 8}}>
                {entries.map(v => 
                    <EntryListItem key={v.id} entry={v} onDelete={() => loadEntries()}/>
                )}
            </Stack>
        </Stack>
        <Route path="/entries/:entry_id" component={EntryId}/>
    </>
}

export default Entries;
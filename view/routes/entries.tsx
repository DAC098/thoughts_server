import { DatePicker, DefaultButton, DetailsList, IColumn, Icon, IconButton, Stack } from "@fluentui/react"
import React, { useEffect, useMemo, useState } from "react"
import { Link, Route, useLocation } from "react-router-dom"
import { json } from "../request"
import EntryId from "./entries/entry_id"
import {EntryJson, getEntries, getMoodFields, MoodFieldJson} from "../json"

const Entries = () => {
    let [fields, setFields] = useState<MoodFieldJson[]>([]);
    let [entries, setEntries] = useState<EntryJson[]>([]);
    let [loading, setLoading] = useState(false);
    let [loading_fields, setLoadingFields] = useState(false);
    let columns = useMemo(() => {
        let rtn: IColumn[] = [
            {
                key: "date",
                name: "Date",
                minWidth: 80,
                maxWidth: 80,
                onRender: (item: EntryJson) => {
                    return <Link to={`/entries/${item.id}`}>
                        {item.created}
                    </Link>
                }
            }
        ];

        for (let field of fields) {
            rtn.push({
                key: field.name,
                name: field.name,
                minWidth: 100,
                maxWidth: 150,
                onRender: field.is_range ?
                    (item: EntryJson) => {
                        for (let m of item.mood_entries) {
                            if (m.field_id === field.id) {
                                return <span>
                                    {`${m.low} - ${m.high} `}
                                    {m.comment && m.comment.length > 0 ?
                                        <Icon iconName="Info"/>
                                        :
                                        null
                                    }
                                </span>
                            }
                        }

                        return <span/>
                    }
                    :
                    (item: EntryJson) => {
                        for (let m of item.mood_entries) {
                            if (m.field_id === field.id) {
                                return <span>
                                    {`${m.low} `}
                                    {m.comment && m.comment.length > 0 ?
                                        <Icon iconName="Info"/>
                                        :
                                        null
                                    }
                                </span>
                            }
                        }

                        return <span/>
                    }
            })
        }

        return rtn;
    }, [fields]);

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

    const loadFields = () => {
        if (loading)
            return;

        setLoadingFields(true);

        getMoodFields().then(list => {
            setFields(list);
        }).catch(err => {
            console.error(err);
        }).then(() => {
            setLoadingFields(false);
        })
    }

    useEffect(() => {
        if (location.pathname !== "/entries") {
            loadEntries();
            loadFields();
        }
    },[]);

    useEffect(() => {
        if (location.pathname === "/entries") {
            loadEntries();
            loadFields();
        }
    },[location.pathname]);

    return <Stack tokens={{padding: 12, childrenGap: 8}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <Stack.Item>
                <Link to="/entries/0">
                    <DefaultButton text="Create Entry" primary/>
                </Link>
            </Stack.Item>
            <Stack.Item>
                <Link to="/mood_fields">
                    <DefaultButton text="Edit Mood Fields"/>
                </Link>
            </Stack.Item>
        </Stack>
        <DetailsList
            items={entries}
            columns={columns}
            compact={true}
        />
    </Stack>
}

export default Entries;
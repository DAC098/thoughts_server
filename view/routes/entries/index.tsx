import { CommandBar, DatePicker, DetailsList, IColumn, Icon, ScrollablePane, Spinner, Stack, Sticky, StickyPositionType, Tooltip, TooltipHost, TooltipOverflowMode } from "@fluentui/react"
import React, { useEffect, useMemo } from "react"
import { Link, useHistory, useParams } from "react-router-dom"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useLoadFields } from "../../hooks/useLoadFields"
import { useOwner } from "../../hooks/useOwner"
import { EntryJson } from "../../api/types"
import { MoodEntryType } from "../../api/mood_entry_types"
import { displayDate, get24hrStr, sameDate } from "../../time"

interface EntriesProps {
    user_specific?: boolean
}

function renderMoodFieldType(value: MoodEntryType) {
    switch (value.type) {
        case "Integer":
            return `${value.value}`
        case "IntegerRange":
            return `${value.low} - ${value.high}`
        case "Float":
            return `${value.value.toFixed(2)}`
        case "FloatRange":
            return `${value.low.toFixed(2)} - ${value.high.toFixed(2)}`
        case "Time":
            return `${displayDate(new Date(value.value))}`;
        case "TimeRange": {
            let low = new Date(value.low);
            let high = new Date(value.high);
            return sameDate(low, high) ? 
                   `${displayDate(low)} - ${get24hrStr(high)}` :
                   `${displayDate(low)} - ${displayDate(high)}`;
        }
    }
}

const Entries = ({user_specific = false}: EntriesProps) => {
    const history = useHistory();
    const params = useParams<{user_id?: string}>();

    const owner = useOwner(user_specific);
    const [entries_state, loadEntries] = useLoadEntries();
    const [mood_fields_state, loadFields] = useLoadFields();

    let columns = useMemo(() => {
        let rtn: IColumn[] = [
            {
                key: "date",
                name: "Date",
                minWidth: 80,
                maxWidth: 160,
                onRender: (item: EntryJson) => {
                    return <Link to={{
                        pathname: `${user_specific ? `/users/${params.user_id}` : ""}/entries/${item.id}`,
                        state: {entry: item}
                    }}>
                        {(new Date(item.created)).toDateString()}
                    </Link>
                }
            }
        ];

        for (let field of mood_fields_state.mood_fields) {
            rtn.push({
                key: field.name,
                name: field.name,
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    for (let m of item.mood_entries) {
                        if (m.field_id === field.id) {
                            let content = <>
                                {renderMoodFieldType(m.value)}
                                {m.comment && m.comment.length > 0 ?
                                    <Icon style={{paddingLeft: 4}} iconName="Info"/>
                                    :
                                    null
                                }
                            </>;

                            return <TooltipHost overflowMode={TooltipOverflowMode.Parent} content={content}>
                                {content}
                            </TooltipHost>
                        }
                    }

                    return <span/>
                }
            })
        }

        return rtn;
    }, [mood_fields_state.mood_fields]);
    
    useEffect(() => {
        if (entries_state.owner !== owner) {
            loadEntries(owner, user_specific);
        }

        if (mood_fields_state.owner !== owner) {
            loadFields(owner, user_specific);
        }
    },[owner]);

    let loading_state = mood_fields_state.loading || entries_state.loading;

    return <Stack style={{
        position: "relative",
        width: "100%",
        height: "100%"
    }}>
        <ScrollablePane>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <Stack tokens={{padding: "8px 8px 0", childrenGap: 8}}>
                    <Stack horizontal tokens={{childrenGap: 8}}>
                        <DatePicker label="From"/>
                        <DatePicker label="To"/>
                    </Stack>
                    <Stack horizontal verticalAlign="center" horizontalAlign="start">
                        <Stack.Item style={{minWidth: 230}}>
                            <CommandBar items={[
                                {
                                    key: "search",
                                    text: "Search",
                                    iconProps: {iconName: "Search"},
                                    onClick: () => loadEntries(owner, user_specific)
                                },
                                {
                                    key: "new_item",
                                    text: "New Entry",
                                    iconProps: {iconName: "Add"},
                                    onClick: () => history.push("/entries/0")
                                }
                            ]}/>
                        </Stack.Item>
                        <div style={{display: loading_state ? null : "none"}}>
                            <Spinner label="loading" labelPosition="right"/>
                        </div>
                    </Stack>
                </Stack>
            </Sticky>
            <DetailsList
                items={entries_state.entries}
                columns={columns}
                compact={true}
                onRenderDetailsHeader={(p,d) => {
                    return <Sticky stickyPosition={StickyPositionType.Header}>
                        {d(p)}
                    </Sticky>
                }}
            />
        </ScrollablePane>
    </Stack>
}

export default Entries;
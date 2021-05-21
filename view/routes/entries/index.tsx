import { CommandBar, DatePicker, Dropdown, IColumn, ICommandBarItemProps, Icon, IconButton, IDropdownOption, ScrollablePane, ShimmeredDetailsList, Spinner, Stack, Sticky, StickyPositionType, TagItem, Tooltip, TooltipHost, TooltipOverflowMode } from "@fluentui/react"
import React, { useEffect, useMemo, useState } from "react"
import { Link, useHistory, useParams } from "react-router-dom"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useLoadFields } from "../../hooks/useLoadFields"
import { useOwner } from "../../hooks/useOwner"
import { EntryJson } from "../../api/types"
import { MoodEntryType } from "../../api/mood_entry_types"
import { diffDates, displayDate, get12hrStr, get24hrStr, sameDate } from "../../time"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { MoodFieldType, Time, TimeRange } from "../../api/mood_field_types"
import { tags_actions } from "../../redux/slices/tags"
import { getBrightness } from "../../util/colors"
import TagToken from "../../components/tags/TagItem"

function renderMoodFieldType(value: MoodEntryType, config: MoodFieldType) {
    switch (value.type) {
        case "Integer":
            return `${value.value}`
        case "IntegerRange":
            return `${value.low} - ${value.high}`
        case "Float":
            return `${value.value.toFixed(2)}`
        case "FloatRange":
            return `${value.low.toFixed(2)} - ${value.high.toFixed(2)}`
        case "Time": {
            return `${displayDate(new Date(value.value), !(config as Time).as_12hr)}`;
        }
        case "TimeRange": {
            let conf = config as TimeRange;
            let low = new Date(value.low);
            let high = new Date(value.high);

            if (conf.show_diff) {
                return diffDates(high, low);
            } else {
                return sameDate(low, high) ? 
                       `${displayDate(low, !conf.as_12hr)} - ${conf.as_12hr ? get12hrStr(high) : get24hrStr(high)}` :
                       `${displayDate(low, !conf.as_12hr)} - ${displayDate(high, !conf.as_12hr)}`;
            }
        }
    }
}

interface EntriesViewProps {
    user_specific?: boolean
}

const EntriesView = ({user_specific = false}: EntriesViewProps) => {
    const history = useHistory();
    const owner = useOwner(user_specific);
    const active_user_state = useAppSelector(state => state.active_user);
    const tags_state = useAppSelector(state => state.tags);
    const appDispatch = useAppDispatch();

    const [entries_state, loadEntries] = useLoadEntries();
    const [mood_fields_state, loadFields] = useLoadFields();

    let [from_date, setFromDate] = useState<Date>(() => {
        if (entries_state.owner != owner)
            return null;
        else
            return entries_state.from != null ? new Date(entries_state.from) : null
    });
    let [to_date, setToDate] = useState<Date>(() => {
        if (entries_state.owner != owner)
            return null;
        else
            return entries_state.to != null ? new Date(entries_state.to) : null
    });
    let [visible_fields, setVisibleFields] = useState<Record<string, boolean>>(() => {
        let rtn = {};

        for (let field of mood_fields_state.mood_fields) {
            rtn[field.name] = true;
        }

        return rtn;
    });

    let columns = useMemo(() => {
        let rtn: IColumn[] = [
            {
                key: "date",
                name: "Date",
                minWidth: 80,
                maxWidth: 160,
                onRender: (item: EntryJson) => {
                    return <Link to={{
                        pathname: `${user_specific ? `/users/${owner}` : ""}/entries/${item.id}`
                    }}>
                        {(new Date(item.created)).toDateString()}
                    </Link>
                }
            }
        ];

        for (let field of mood_fields_state.mood_fields) {
            if (!visible_fields[field.name]) {
                continue;
            }

            rtn.push({
                key: field.name,
                name: field.name,
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    for (let m of item.mood_entries) {
                        if (m.field_id === field.id) {
                            let content = <>
                                {renderMoodFieldType(m.value, mood_fields_state.mapping[m.field_id].config)}
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

        if (tags_state.tags.length > 0) {
            rtn.push({
                key: "tags",
                name: "Tags",
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    let content = [];
    
                    for (let tag of item.tags) {
                        let title = tags_state.mapping[tag].title;
                        let color = tags_state.mapping[tag].color;
    
                        content.push(<TagToken 
                            key={tag} color={color} title={title} 
                            fontSize={null} lineHeight={20}
                        />);
                    }
    
                    return <TooltipHost overflowMode={TooltipOverflowMode.Parent} content={content} children={content}/>
                }
            })
        }

        return rtn;
    }, [mood_fields_state.mood_fields, tags_state.tags, visible_fields]);

    useEffect(() => {
        let rtn = {};

        for (let field of mood_fields_state.mood_fields) {
            rtn[field.name] = true;
        }
        
        setVisibleFields(rtn);
    }, [mood_fields_state.mood_fields])
    
    useEffect(() => {
        if (entries_state.owner !== owner) {
            loadEntries(owner, user_specific, {from: from_date, to: to_date});
        }

        if (mood_fields_state.owner !== owner) {
            loadFields(owner, user_specific);
        }

        if (tags_state.owner !== owner) {
            appDispatch(tags_actions.fetchTags({owner, user_specific}));
        }
    }, [owner]);

    let loading_state = mood_fields_state.loading || entries_state.loading;
    let command_bar_actions = [
        {
            key: "search",
            text: "Search",
            iconProps: {iconName: "Search"},
            onClick: () => loadEntries(owner, user_specific, {from: from_date, to: to_date})
        }
    ];

    if (active_user_state.user.id === owner) {
        command_bar_actions.push({
            key: "new_item",
            text: "New Entry",
            iconProps: {iconName: "Add"},
            onClick: () => history.push("/entries/0")
        });
    }

    let visible_fields_options: ICommandBarItemProps[] = [];

    for (let field of mood_fields_state.mood_fields) {
        visible_fields_options.push({
            key: field.name, 
            text: field.name,
            canCheck: true,
            checked: visible_fields[field.name]
        });
    }

    return <Stack style={{
        position: "relative",
        width: "100%",
        height: "100%"
    }}>
        <ScrollablePane styles={{"root": {}}}>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                <Stack tokens={{padding: "8px 8px 0", childrenGap: 8}}>
                    <Stack horizontal tokens={{childrenGap: 8}}>
                        <Stack horizontal  verticalAlign="end">
                            <DatePicker label="From" value={from_date} onSelectDate={d => {
                                setFromDate(d);
                            }}/>
                            <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                                setFromDate(null);
                            }}/>
                        </Stack>
                        <Stack horizontal verticalAlign="end">
                            <DatePicker label="To" value={to_date} onSelectDate={d => {
                                setToDate(d);
                            }}/>
                            <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                                setToDate(null);
                            }}/>
                        </Stack>
                    </Stack>
                    <CommandBar 
                        items={command_bar_actions}
                        farItems={[
                            {
                                key: "settings", 
                                text: "Settings",
                                iconOnly: true,
                                iconProps: {iconName: "Settings"},
                                subMenuProps: {
                                    items: [
                                        {
                                            key: "fields",
                                            text: "Fields",
                                            disabled: mood_fields_state.mood_fields.length === 0,
                                            subMenuProps: {
                                                onItemClick: (ev, item) => {
                                                    ev.preventDefault();

                                                    setVisibleFields(v => ({
                                                        ...v,
                                                        [item.key]: !visible_fields[item.key]
                                                    }));
                                                },
                                                items: visible_fields_options
                                            }
                                        }
                                    ]
                                }
                            }
                        ]}
                    />
                </Stack>
            </Sticky>
            <ShimmeredDetailsList
                items={loading_state ? [] : entries_state.entries}
                columns={columns}
                compact={false}
                enableShimmer={loading_state}
                onRenderDetailsHeader={(p,d) => {
                    return <Sticky stickyPosition={StickyPositionType.Header}>
                        {d(p)}
                    </Sticky>
                }}
            />
        </ScrollablePane>
    </Stack>
}

export default EntriesView;
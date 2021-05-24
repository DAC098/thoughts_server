import { CommandBar, DatePicker, IColumn, ICommandBarItemProps, Icon, IconButton, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType, TooltipHost, TooltipOverflowMode } from "@fluentui/react"
import React, { useEffect, useMemo, useState } from "react"
import { Link, useHistory } from "react-router-dom"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useLoadFields } from "../../hooks/useLoadFields"
import { useOwner } from "../../hooks/useOwner"
import { EntryJson } from "../../api/types"
import { CustomFieldEntryType } from "../../api/custom_field_entry_types"
import { diffDates, displayDate, get12hrStr, get24hrStr, sameDate } from "../../time"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { CustomFieldType, Float, FloatRange, Time, TimeRange } from "../../api/custom_field_types"
import { tags_actions } from "../../redux/slices/tags"
import TagToken from "../../components/tags/TagItem"

function renderMoodFieldType(value: CustomFieldEntryType, config: CustomFieldType) {
    switch (value.type) {
        case "Integer":
            return `${value.value}`;
        case "IntegerRange":
            return `${value.low} - ${value.high}`;
        case "Float": {
            let conf = config as Float;
            return `${value.value.toFixed(conf.precision)}`;
        }
        case "FloatRange": {
            let conf = config as FloatRange;
            return `${value.low.toFixed(conf.precision)} - ${value.high.toFixed(conf.precision)}`;
        }
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
    const [custom_fields_state, loadFields] = useLoadFields();

    const [from_date, setFromDate] = useState<Date>(() => {
        if (entries_state.owner != owner)
            return null;
        else
            return entries_state.from != null ? new Date(entries_state.from) : null
    });
    const [to_date, setToDate] = useState<Date>(() => {
        if (entries_state.owner != owner)
            return null;
        else
            return entries_state.to != null ? new Date(entries_state.to) : null
    });
    const [visible_fields, setVisibleFields] = useState<Record<string, boolean>>(() => {
        let rtn = {};

        for (let field of custom_fields_state.custom_fields) {
            rtn[field.name] = true;
        }

        return rtn;
    });

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

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

        for (let field of custom_fields_state.custom_fields) {
            if (!visible_fields[field.name]) {
                continue;
            }

            rtn.push({
                key: field.name,
                name: field.name,
                minWidth: 100,
                maxWidth: 150,
                onRender: (item: EntryJson) => {
                    for (let m of item.custom_field_entries) {
                        if (m.field === field.id) {
                            let content = <>
                                {renderMoodFieldType(m.value, custom_fields_state.mapping[m.field].config)}
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
    }, [loading_state, visible_fields]);

    useEffect(() => {
        let rtn = {};

        for (let field of custom_fields_state.custom_fields) {
            rtn[field.name] = true;
        }
        
        setVisibleFields(rtn);
    }, [custom_fields_state.custom_fields])
    
    useEffect(() => {
        if (entries_state.owner !== owner) {
            loadEntries(owner, user_specific, {from: from_date, to: to_date});
        }

        if (custom_fields_state.owner !== owner) {
            loadFields(owner, user_specific);
        }

        if (tags_state.owner !== owner) {
            appDispatch(tags_actions.fetchTags({owner, user_specific}));
        }
    }, [owner]);

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

    for (let field of custom_fields_state.custom_fields) {
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
        <ScrollablePane styles={{"contentContainer": {height: "100%"}}}>
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
                                            disabled: custom_fields_state.custom_fields.length === 0,
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
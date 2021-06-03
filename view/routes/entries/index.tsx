import { CommandBar, DatePicker, Dropdown, IColumn, ICommandBarItemProps, Icon, IconButton, IContextualMenuItem, ScrollablePane, ShimmeredDetailsList, Stack, Sticky, StickyPositionType, Text, TooltipHost, TooltipOverflowMode } from "@fluentui/react"
import React, { useEffect, useMemo, useState } from "react"
import { Link, useHistory } from "react-router-dom"
import ParentSize from "@visx/responsive/lib/components/ParentSize"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useLoadFields } from "../../hooks/useLoadFields"
import { useOwner } from "../../hooks/useOwner"
import { CustomFieldJson, EntryJson } from "../../api/types"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { tags_actions } from "../../redux/slices/tags"
import TagToken from "../../components/tags/TagItem"
import { downloadLink } from "../../util/downloadLink"
import { getURL } from "../../api"
import { CustomFieldGraph } from "../../components/graphs"
import { CustomFieldEntryCell } from "../../components/CustomFieldEntryCell"
import { common_ratios, containRatio } from "../../util/math"

interface EntriesGraphViewProps {
    field: CustomFieldJson
    entries: EntryJson[]
}

const EntriesGraphView = ({field, entries}: EntriesGraphViewProps) => {
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const entries_state = useAppSelector(state => state.entries);
    const tags_state = useAppSelector(state => state.tags);

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    return <ParentSize 
        className="ms-StackItem"
        debounceTime={20}
        parentSizeStyles={{
            width: "auto", height: "400px", 
            flexGrow: 1, flexShrink: 1, 
            position: "relative", 
            overflow: "hidden"
        }}
    >
        {({width: w, height: h}) => {
            let {width, height} = containRatio(w, h, common_ratios.r_16_9);

            return loading_state ? null :
                <CustomFieldGraph 
                    field={field} entries={entries} 
                    width={width} height={height}
                />
            }
        }
    </ParentSize>
}

interface EntriesTableViewProps {
    user_specific: boolean
    owner: number
    visible_fields: Record<string, boolean>
}

const EntriesTableView = ({user_specific, owner, visible_fields}: EntriesTableViewProps) => {
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const entries_state = useAppSelector(state => state.entries);
    const tags_state = useAppSelector(state => state.tags);

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
                    let custom_field_entry = item.custom_field_entries?.[field.id];

                    if (!custom_field_entry) {
                        return <span/>
                    }

                    let content = <>
                        <CustomFieldEntryCell value={custom_field_entry.value} config={field.config}/>
                        {custom_field_entry.comment && custom_field_entry.comment.length > 0 ?
                            <Icon style={{paddingLeft: 4}} iconName="Info"/>
                            :
                            null
                        }
                    </>;

                    return <TooltipHost overflowMode={TooltipOverflowMode.Parent} content={content}>
                        {content}
                    </TooltipHost>
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

    return <ShimmeredDetailsList
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

    const [view_graph, setViewGraph] = useState(false);
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
    const [selected_field, setSelectedField] = useState<CustomFieldJson>(null);

    const owner_is_active_user = active_user_state.user.id === owner;
    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    const default_download = () => {
        let url = getURL("/backup");
        let filename = [];

        if (from_date != null) {
            url.searchParams.append("from", from_date.toISOString());
            filename.push(from_date.toDateString());
        } else {
            filename.push("");
        }

        if (to_date != null) {
            url.searchParams.append("to", to_date.toISOString());
            filename.push(to_date.toDateString());
        } else {
            filename.push("");
        }

        downloadLink(url.toString(), filename.join("_to_") + ".json");
    }

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

    let download_options: IContextualMenuItem[] = [];
    let command_bar_actions: ICommandBarItemProps[] = [
        {
            key: "search",
            text: "Search",
            iconProps: {iconName: "Search"},
            onClick: () => loadEntries(owner, user_specific, {from: from_date, to: to_date})
        }
    ];

    if (owner_is_active_user) {
        command_bar_actions.push({
            key: "new_item",
            text: "New Entry",
            iconProps: {iconName: "Add"},
            onClick: () => history.push("/entries/0")
        });

        download_options.push({
            key: "json",
            text: "JSON",
            onClick: default_download
        });
    }

    command_bar_actions.push({
        key: "download",
        text: "Download",
        split: owner_is_active_user,
        iconProps: {iconName: "Download"},
        onClick: owner_is_active_user ? default_download : null,
        disabled: download_options.length === 0,
        subMenuProps: {
            items: download_options
        }
    });

    if (owner_is_active_user) {
        command_bar_actions.push({
            key: "upload",
            text: "Upload",
            iconProps: {iconName: "Upload"},
            disabled: true,
            onClick: () => {}
        })
    }

    let visible_fields_options: ICommandBarItemProps[] = [];

    if (!view_graph) {
        for (let field of custom_fields_state.custom_fields) {
            visible_fields_options.push({
                key: field.name, 
                text: field.name,
                canCheck: true,
                checked: visible_fields[field.name]
            });
        }
    }

    let settings_submenus: IContextualMenuItem[] = [
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
    ];

    const header_content = <Stack tokens={{padding: "8px 8px 0", childrenGap: 8}}>
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
            {view_graph ?
                <Dropdown
                    label="Graph Field"
                    styles={{root: {width: 200}}}
                    options={custom_fields_state.custom_fields.map(field => {
                        return {
                            key: field.id,
                            text: field.name,
                            selected: selected_field ? selected_field.id === field.id : false,
                            data: field
                        }
                    })}
                    onChange={(e, o) => {
                        setSelectedField(o.data);
                    }}
                />
                :
                null
            }
        </Stack>
        <Text variant="smallPlus">{!loading_state ? `${entries_state.entries.length} total entries` : "loading"}</Text>
        <CommandBar 
            items={command_bar_actions}
            farItems={[
                {
                    key: "view_change",
                    text: view_graph ? "Table View" : "Graph View",
                    iconOnly: true,
                    iconProps: {iconName: view_graph ? "Table" : "LineChart"},
                    onClick: () => setViewGraph(!view_graph)
                },
                {
                    key: "settings", 
                    text: "Settings",
                    iconOnly: true,
                    iconProps: {iconName: "Settings"},
                    subMenuProps: {
                        items: settings_submenus
                    }
                }
            ]}
        />
    </Stack>;

    return <Stack style={{
        position: "relative",
        width: "100%",
        height: "100%"
    }}>
        {view_graph ?
            <>
                {header_content}
                {!loading_state && selected_field != null ?
                    <EntriesGraphView field={selected_field} entries={entries_state.entries}/>
                    :
                    null
                }
            </>
            :
            <ScrollablePane styles={{"contentContainer": {height: "100%"}}}>
                <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor={"white"}>
                    {header_content}
                </Sticky>
                <EntriesTableView user_specific={user_specific} owner={owner} visible_fields={visible_fields}/>
            </ScrollablePane>
        }
    </Stack>
}

export default EntriesView;
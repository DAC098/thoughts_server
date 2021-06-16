import React, { useContext, useState } from "react"
import { CommandBar, DatePicker, Dropdown, ICommandBarItemProps, IconButton, IContextualMenuItem, ScrollablePane, Stack, Sticky, StickyPositionType, Text } from "@fluentui/react"
import { useHistory } from "react-router-dom"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useAppSelector } from "../../hooks/useApp"
import { downloadLink } from "../../util/downloadLink"
import { getURL } from "../../api"
import { EntriesViewContext, EntriesViewState, entries_view_actions } from "./reducer"

export interface CommandBarViewProps {
    owner: number
    user_specific: boolean

    entries_view_state: EntriesViewState
}

export function CommandBarView({
    owner, user_specific, 
    entries_view_state
}: CommandBarViewProps) {
    const history = useHistory();
    const active_user_state = useAppSelector(state => state.active_user);
    const tags_state = useAppSelector(state => state.tags);
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const dispatch = useContext(EntriesViewContext);
    
    const loadEntries = useLoadEntries();

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

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;
    const owner_is_active_user = active_user_state.user.id === owner;

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

    if (!entries_view_state.view_graph) {
        for (let field of custom_fields_state.custom_fields) {
            visible_fields_options.push({
                key: field.name, 
                text: field.name,
                canCheck: true,
                checked: entries_view_state.visible_fields[field.name]
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

                    dispatch(entries_view_actions.toggle_visible_field(item.key));
                },
                items: visible_fields_options
            }
        }
    ];

    return <Stack tokens={{padding: "8px 8px 0", childrenGap: 8}}>
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
            {entries_view_state.view_graph ?
                <Dropdown
                    label="Graph Field"
                    styles={{root: {width: 200}}}
                    options={custom_fields_state.custom_fields.map(field => {
                        return {
                            key: field.id,
                            text: field.name,
                            selected: entries_view_state.selected_field ? entries_view_state.selected_field.id === field.id : false,
                            data: field
                        }
                    })}
                    onChange={(e, o) => {
                        dispatch(entries_view_actions.set_selected_field(o.data));
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
                    text: entries_view_state.view_graph ? "Table View" : "Graph View",
                    iconOnly: true,
                    iconProps: {iconName: entries_view_state.view_graph ? "Table" : "LineChart"},
                    onClick: () => dispatch(entries_view_actions.toggle_view_graph())
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
}
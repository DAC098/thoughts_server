import React, { useContext, useState, useEffect } from "react"
import { CommandBar, DatePicker, Dropdown, ICommandBarItemProps, IconButton, IContextualMenuItem, IDropdownOption, ScrollablePane, Stack, Sticky, StickyPositionType, Text } from "@fluentui/react"
import { useHistory, useLocation } from "react-router-dom"
import { useLoadEntries } from "../../hooks/useLoadEntries"
import { useAppDispatch, useAppSelector } from "../../hooks/useApp"
import { downloadLink } from "../../util/downloadLink"
import { getURL } from "../../api"
import { EntriesViewContext, EntriesViewState, entries_view_actions } from "./reducer"
import { tags_actions } from "../../redux/slices/tags"
import { useLoadFields } from "../../hooks/useLoadFields"
import { noOriginUrlString, stringFromLocation, urlFromLocation } from "../../util/url"
import { dateFromUnixTime, unixTimeFromDate } from "../../util/time"

function getFromDate(url: URL) {
    if (url.searchParams.has("from")) {
        try {
            let num = parseInt(url.searchParams.get("from"), 10);

            if (isNaN(num)) {
                return null;
            }

            return dateFromUnixTime(num);
        } catch(err) {}
    }

    return null;
}

function getToDate(url: URL) {
    if (url.searchParams.has("to")) {
        try {
            let num = parseInt(url.searchParams.get("to"), 10);

            if (isNaN(num)) {
                return null;
            }

            return dateFromUnixTime(num);
        } catch(err) {}
    }

    return null;
}

function getDates(url: URL) {
    return {
        from: getFromDate(url),
        to: getToDate(url)
    }
}

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
    const location = useLocation();
    const active_user_state = useAppSelector(state => state.active_user);
    const tags_state = useAppSelector(state => state.tags);
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const appDispatch = useAppDispatch();
    const dispatch = useContext(EntriesViewContext);
    
    const loadEntries = useLoadEntries();
    const loadFields = useLoadFields();

    const [from_date, setFromDate] = useState<Date>(getFromDate(urlFromLocation(location)));
    const [to_date, setToDate] = useState<Date>(getToDate(urlFromLocation(location)));
    const [tags_selected, setTagsSelected] = useState<number[]>(() => {
        let url = new URL(location.pathname + location.search + location.hash, window.location.origin);
        return []
    });

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

    useEffect(() => {
        const onPopState = (ev: PopStateEvent) => {
            const location = document.location;
            // ignore anything that is not an entries path
            if (!location.pathname.endsWith("entries")) {
                return;
            }

            let url = urlFromLocation(location);
            let dates = getDates(url);

            setFromDate(dates.from);
            setToDate(dates.to);
            loadEntries(owner, user_specific, {from: dates.from, to: dates.to});
        };

        window.addEventListener("popstate", onPopState);

        return () => {
            window.removeEventListener("popstate", onPopState);
        }
    }, []);

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;
    const owner_is_active_user = active_user_state.user.id === owner;

    const default_download = () => {
        let url = getURL("/backup");
        let filename = [];

        if (from_date != null) {
            url.searchParams.append("from", unixTimeFromDate(from_date).toString());
            filename.push(from_date.toDateString());
        } else {
            filename.push("");
        }

        if (to_date != null) {
            url.searchParams.append("to", unixTimeFromDate(to_date).toString());
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
            onClick: () => {
                let url = new URL(location.pathname, window.location.origin);

                if (from_date != null) {
                    url.searchParams.append("from", unixTimeFromDate(from_date).toString());
                }

                if (to_date != null) {
                    url.searchParams.append("to", unixTimeFromDate(to_date).toString());
                }

                history.push(noOriginUrlString(url));
                loadEntries(owner, user_specific, {from: from_date, to: to_date});
            }
        }
    ];

    if (owner_is_active_user) {
        command_bar_actions.push({
            key: "new_item",
            text: "New Entry",
            iconProps: {iconName: "Add"},
            onClick: () => {
                history.push(
                    `/entries/0?prev=${stringFromLocation(window.location, {encode: true, decode_search: true})}`
                );
            }
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

    return <Stack tokens={{childrenGap: 8}}>
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
                <Dropdown
                    label={"Tags"}
                    selectedKeys={tags_selected}
                    onChange={(e, o) =>
                        setTagsSelected(o.selected ? [...tags_selected, (o.key as number)] : tags_selected.filter(v => v !== o.key))
                    }
                    multiSelect
                    options={tags_state.tags.map<IDropdownOption>(v => {
                        return {
                            key: v.id,
                            text: v.title,
                        }
                    })}
                    styles={{"root": {width: 200}}}
                />
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
        </Stack>
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
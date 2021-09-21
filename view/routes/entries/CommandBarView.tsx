import React, { useContext, useState, useEffect } from "react"
import { CommandBar, DatePicker, Dropdown, ICommandBarItemProps, IconButton, IContextualMenuItem, IDropdownOption, Label, ScrollablePane, Stack, Sticky, StickyPositionType, Text } from "@fluentui/react"
import { useHistory, useLocation } from "react-router-dom"
import useAppDispatch from "../../hooks/useAppDispatch"
import useAppSelector from "../../hooks/useAppSelector"
import { downloadLink } from "../../util/downloadLink"
import { EntriesViewContext, EntriesViewState, entries_view_actions } from "./reducer"
import { tags_actions } from "../../redux/slices/tags"
import { useGlobalFetchCustomFields } from "../../hooks/useGlobalFetchCustomFields"
import { noOriginUrlString, stringFromLocation, urlFromLocation, urlFromString } from "../../util/url"
import { dateFromUnixTime, unixTimeFromDate } from "../../util/time"
import { entries_actions } from "../../redux/slices/entries"
import { useGlobalFetchEntries } from "../../hooks/useGlobalFetchEntries"
import { useGlobalFetchTags } from "../../hooks/useGlobalFetchTags"

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
    user_id?: number | string

    entries_view_state: EntriesViewState
}

export function CommandBarView({
    user_id,
    entries_view_state
}: CommandBarViewProps) {
    const tags_state = useAppSelector(state => state.tags);
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);

    const history = useHistory();
    const location = useLocation();
    const appDispatch = useAppDispatch();
    const dispatch = useContext(EntriesViewContext);
    
    const globalFetchCustomFields = useGlobalFetchCustomFields();
    const globalFetchEntries = useGlobalFetchEntries();
    const globalFetchTags = useGlobalFetchTags();

    const [from_date, setFromDate] = useState<Date>(getFromDate(urlFromLocation(location)));
    const [to_date, setToDate] = useState<Date>(getToDate(urlFromLocation(location)));
    const [tags_selected, setTagsSelected] = useState<number[]>(() => {
        let url = new URL(location.pathname + location.search + location.hash, window.location.origin);
        return []
    });

    useEffect(() => {
        if (entries_state.owner !== user_id) {
            globalFetchEntries({user_id, query: {from: from_date, to: to_date}});
        }

        if (custom_fields_state.owner !== user_id) {
            globalFetchCustomFields({user_id});
        }

        if (tags_state.owner !== user_id) {
            globalFetchTags({user_id});
        }
    }, [user_id]);

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
            appDispatch(entries_actions.fetchEntries({
                user_id, query: {from: dates.from, to: dates.to}
            }));
        };

        window.addEventListener("popstate", onPopState);

        return () => {
            window.removeEventListener("popstate", onPopState);
        }
    }, []);

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;
    const viewing_another_user =  user_id != null;

    const default_download = () => {
        let url = urlFromString("/backup");
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
            disabled: entries_state.loading,
            onClick: () => {
                let url = new URL(location.pathname, window.location.origin);

                if (from_date != null) {
                    url.searchParams.append("from", unixTimeFromDate(from_date).toString());
                }

                if (to_date != null) {
                    url.searchParams.append("to", unixTimeFromDate(to_date).toString());
                }

                history.push(noOriginUrlString(url));
                appDispatch(entries_actions.fetchEntries({user_id, query: {from: from_date, to: to_date}}));
            }
        }
    ];

    if (!viewing_another_user) {
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
        split: !viewing_another_user,
        iconProps: {iconName: "Download"},
        onClick: !viewing_another_user ? default_download : null,
        disabled: download_options.length === 0,
        subMenuProps: {
            items: download_options
        }
    });

    if (!viewing_another_user) {
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
                <Stack>
                    <Label htmlFor="from-date-selector">From</Label>
                    <Stack horizontal>
                        <DatePicker id="from-date-selector" value={from_date} onSelectDate={d => setFromDate(d)}/>
                        <IconButton iconProps={{iconName: "Delete"}} onClick={() => setFromDate(null)}/>
                    </Stack>
                </Stack>
                <Stack>
                    <Label htmlFor="to-date-selector">To</Label>
                    <Stack horizontal>
                        <DatePicker id="to-date-selector" value={to_date} onSelectDate={d => setToDate(d)}/>
                        <IconButton iconProps={{iconName: "Delete"}} onClick={() => setToDate(null)}/>
                    </Stack>
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
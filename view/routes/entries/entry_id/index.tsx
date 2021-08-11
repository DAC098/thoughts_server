import { ContextualMenuItemType, DatePicker, DefaultButton, Dialog, DialogFooter, DialogType, IBasePicker, IconButton, IContextualMenuItem, ITag, Label, ScrollablePane, Separator, Stack, Sticky, StickyPositionType, TagItem, TagItemSuggestion, TagPicker, Text, TextField, Toggle } from "@fluentui/react"
import React, { Fragment, useContext, useEffect, useReducer, useRef } from "react"
import { useHistory, useLocation, useParams } from "react-router-dom"
import api from "../../../api"
import { CustomFieldEntryType } from "../../../api/custom_field_entry_types"
import { ComposedEntry, CustomFieldEntry, CustomField, Tag, TextEntry } from "../../../api/types"
import { CustomFieldEntryTypeEditView, CustomFieldEntryTypeReadView } from "../../../components/custom_field_entries"
import { useAppDispatch, useAppSelector } from "../../../hooks/useApp"
import { entryIdViewSlice, EntryIdViewContext, initialState, TextEntryUI, entry_id_view_actions, EntryIdViewReducer, EntryMarkerUI } from "./reducer"
import { actions as entries_actions } from "../../../redux/slices/entries"
import arrayFilterMap from "../../../util/arrayFilterMap"
import { getBrightness } from "../../../util/colors"
import TagToken from "../../../components/tags/TagItem"
import OverlayedPage from "../../../components/OverlayedPage"
import { stringFromLocation, urlFromLocation } from "../../../util/url"
import { dateFromUnixTime, unixTimeFromDate } from "../../../util/time"

interface TextEntryEditViewProps {
    text_entries: TextEntryUI[]
}

const TextEntryEditView = ({text_entries}: TextEntryEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v, index) => {
            return <Stack key={v.key ?? v.id} tokens={{childrenGap: 8}}>
                <TextField multiline autoAdjustHeight value={v.thought} onChange={(e, thought) => {
                    dispatch(entry_id_view_actions.update_text_entry({index, thought, private: v.private}));
                }}/>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Toggle label="Private" inlineLabel onText="Yes" offText="No" checked={v.private} onChange={(e,checked) => {
                        dispatch(entry_id_view_actions.update_text_entry({index, thought: v.thought, private: checked}))
                    }}/>
                    <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                        dispatch(entry_id_view_actions.delete_text_entry(index));
                    }}/>
                </Stack>
            </Stack>
        })}
    </Stack>
}

interface TextEntryReadViewProps {
    text_entries: TextEntry[]
}

const TextEntryReadView = ({text_entries}: TextEntryReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v) => {
            let line_splits = v.thought.split(/\n/);
            let total = line_splits.length;

            return <div key={v.id}>
                {line_splits.map((t, i) =>
                    <div key={i} style={{paddingBottom: total - 1 !== i ? 4 : 0}}>
                        <Text>{t}</Text>
                    </div>
                )}
            </div>
        })}
    </Stack>
}

interface CustomFieldEntryInputProps {
    field: CustomField
    entry: CustomFieldEntry

    onDelete?: () => void
    onChange?: (entry: {comment: string, value: CustomFieldEntryType}) => void
}

const CustomFieldEntryInputs = ({
    field,
    entry,
    onDelete,
    onChange
}: CustomFieldEntryInputProps) => {
    let similar_types = entry.value.type === field.config.type;

    return <Stack tokens={{childrenGap: 8}}>
        <Separator alignContent="start">
            <Stack horizontal tokens={{childrenGap: 8}}>
                <Label title={field.comment}>{field.name}</Label>
                <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                    onDelete?.();
                }}/>
            </Stack>
        </Separator>
        <CustomFieldEntryTypeEditView value={entry.value} config={similar_types ? field.config : null} onChange={value => {
            onChange?.({comment: entry.comment, value});
        }}/>
        <TextField type="text" placeholder="comment" value={entry.comment ?? ""} onChange={(e,v) => {
            onChange?.({comment: v, value: entry.value});
        }}/>
    </Stack>
}

interface CustomFieldEntriesEditViewProps {
    custom_fields: CustomField[]
    custom_field_entries: {[id: string]: CustomFieldEntry}
}

const CustomFieldEntriesEditView = ({custom_fields, custom_field_entries}: CustomFieldEntriesEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {custom_fields.filter(field => field.id in custom_field_entries).map((field, index) =>
            <CustomFieldEntryInputs
                key={field.id}
                field={field}
                entry={custom_field_entries[field.id]}
        
                onDelete={() => dispatch(entry_id_view_actions.delete_mood_entry(field.id.toString()))}
                onChange={(value) => dispatch(entry_id_view_actions.update_mood_entry({index: field.id, ...value}))}
            />
        )}
    </Stack>
}

interface CustomFieldEntriesReadViewProps {
    custom_field_entries: {[id: string]: CustomFieldEntry}
    custom_fields: CustomField[]
}

const CustomFieldEntriesReadView = ({custom_fields, custom_field_entries}: CustomFieldEntriesReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {custom_fields.filter(field => field.id in custom_field_entries).map((field) => 
            <Stack key={field.id} tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <Separator alignContent="start">
                        <Label title={field.comment}>{field.name}</Label>
                    </Separator>
                    <CustomFieldEntryTypeReadView 
                        value={custom_field_entries[field.id].value}
                        config={field.config}
                    />
                </Stack>
                <Text>{custom_field_entries[field.id].comment}</Text>
            </Stack>
        )}
    </Stack>
}

interface EntryMarkerEditViewProps {
    markers: EntryMarkerUI[]
}

const EntryMarkerEditView = ({markers}: EntryMarkerEditViewProps) => {
    const dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {markers.length > 0 ?
            <Separator alignContent="start">
                <Label>Markers</Label>
            </Separator>
            :
            null
        }
        {markers.map((marker, index) =>
            <Stack key={marker.id ?? marker.key} horizontal tokens={{childrenGap: 8}} verticalAlign="end">
                <TextField
                    label="Title"
                    type="text"
                    value={marker.title}
                    onChange={(e, v) => 
                        dispatch(entry_id_view_actions.update_entry_marker({index, title: v, comment: marker.comment}))
                    }
                />
                <TextField
                    label="Comment"
                    type="text"
                    value={marker.comment ?? ""}
                    styles={{root: {flex: 1}}}
                    onChange={(e, v) =>
                        dispatch(entry_id_view_actions.update_entry_marker({index, title: marker.title, comment: v}))
                    }
                />
                <IconButton iconProps={{iconName: "Delete"}} onClick={() =>
                    dispatch(entry_id_view_actions.delete_entry_marker(index))
                }/>
            </Stack>
        )}
    </Stack>
}

interface EntryMarkerReadViewProps {
    markers: EntryMarkerUI[]
}

const EntryMarkerReadView = ({markers}: EntryMarkerReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {markers.length > 0 ?
            <Separator alignContent="start">
                <Label>Markers</Label>
            </Separator>
            :
            null
        }
        {markers.map((marker) =>
            <Stack key={marker.id ?? marker.key} horizontal tokens={{childrenGap: 8}} verticalAlign="end">
                <Text>{marker.title}</Text>
                <Text variant="small">{marker.comment?.length ? marker.comment : ""}</Text>
            </Stack>
        )}
    </Stack>
}

interface EntryIdProps {
    user_specific?: boolean
}

const EntryId = ({user_specific = false}: EntryIdProps) => {
    const location = useLocation();
    const history = useHistory();
    const params = useParams<{entry_id: string, user_id?: string}>();
    
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const tags_state = useAppSelector(state => state.tags);
    const appDispatch = useAppDispatch();

    const allow_edit = params.user_id == null;

    const tag_picker = useRef<IBasePicker<ITag>>(null);
    let [state, dispatch] = useReducer<EntryIdViewReducer>(entryIdViewSlice.reducer, initialState(allow_edit, params));

    const fetchEntry = () => {
        if (state.loading) {
            return;
        }

        dispatch(entry_id_view_actions.set_loading(true));

        (user_specific ?
            api.users.id.entries.id.get(params.user_id, params.entry_id) :
            api.entries.id.get(params.entry_id)
        ).then(entry => {
            dispatch(entry_id_view_actions.set_entry(entry));
        }).catch(console.error).then(() => {
            dispatch(entry_id_view_actions.set_loading(false));
        })
    }

    const sendEntry = () => {
        if (user_specific)
            return;

        if (state.current == null)
            return;
        
        if (state.sending)
            return;

        dispatch(entry_id_view_actions.set_sending(true));

        let promise = null;
        
        if (state.current.entry.id) {
            promise = api.entries.id.put(state.current.entry.id, {
                entry: {
                    day: state.current.entry.day
                },
                tags: state.current.tags,
                markers: state.current.markers,
                custom_field_entries: Object.values(
                    state.current.custom_field_entries
                    ).map(v => ({
                        field: v.field,
                        value: v.value,
                        comment: v.comment
                    })),
                text_entries: state.current.text_entries.map(v => ({
                    id: v.id, thought: v.thought, private: v.private}
                ))
            }).then(entry => {
                dispatch(entryIdViewSlice.actions.set_entry(entry));
                appDispatch(entries_actions.update_entry(entry));
            })
        } else {
            promise = api.entries.post({
                entry: {
                    day: state.current.entry.day
                },
                tags: state.current.tags,
                markers: state.current.markers,
                custom_field_entries: Object.values(
                    state.current.custom_field_entries
                    ).map(v => ({
                        field: v.field,
                        value: v.value,
                        comment: v.comment
                    })),
                text_entries: state.current.text_entries.map(v => ({
                    thought: v.thought, private: v.private
                }))
            }).then(entry => {
                let base_path = location.pathname.split("/");
                base_path.pop();
                base_path.push(entry.entry.id.toString());

                history.push(stringFromLocation({
                    ...location, 
                    pathname: base_path.join("/")
                }));
                dispatch(entry_id_view_actions.set_entry(entry));
                appDispatch(entries_actions.add_entry(entry));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch(entry_id_view_actions.set_sending(false));
        });
    }

    const deleteEntry = () => {
        if (user_specific) {
            return;
        }

        if (state.current == null || state.current.entry.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch(entry_id_view_actions.set_deleting(true));

        api.entries.id.del(state.current.entry.id).then(() => {
            appDispatch(entries_actions.delete_entry(state.current.entry.id));
            let url = urlFromLocation(location);
            
            if (url.searchParams.has("prev")) {
                history.push(url.searchParams.get("prev"));
            } else {
                let new_path = location.pathname.split("/");
                new_path.pop();
                
                history.push(new_path.join("/"));
            }
        }).catch((e) => {
            console.error(e);
            dispatch(entry_id_view_actions.set_deleting(false));
        });
    }

    useEffect(() => {
        let entry_id = parseInt(params.entry_id);

        if (isNaN(entry_id) || entry_id === 0) {
            dispatch(entry_id_view_actions.new_entry());
            return;
        }
        
        if (entries_state.loading) {
            fetchEntry();
        } else {
            for (let entry of entries_state.entries) {
                if (entry.entry.id === entry_id) {
                    dispatch(entry_id_view_actions.set_entry(entry));
                    return;
                }
            }

            fetchEntry();
        }
    }, [params.entry_id]);

    let fields_section = [];
    let issued_fields_section = [];

    for (let field of custom_fields_state.custom_fields) {
        if (state.current && field.id in state.current?.custom_field_entries) {
            continue;
        }

        if (custom_fields_state.mapping[field.id].issued_by != null) {
            issued_fields_section.push({
                key: custom_fields_state[field.id].name,
                text: custom_fields_state[field.id].name,
                title: custom_fields_state[field.id].comment,
                onClick: () => {
                    dispatch(entry_id_view_actions.create_mood_entry(field.id.toString()))
                }
            });
        } else {
            fields_section.push({
                key: custom_fields_state.mapping[field.id].name,
                text: custom_fields_state.mapping[field.id].name,
                title: custom_fields_state.mapping[field.id].comment,
                onClick: () => {
                    dispatch(entry_id_view_actions.create_mood_entry(field.id.toString()))
                }
            });
        }
    }

    let entry_options: IContextualMenuItem[] = [
        {
            key: "new-text-entry", 
            text: "Text Entry", 
            onClick: () => {
                dispatch(entry_id_view_actions.create_text_entry());
            }
        },
        {
            key: "new-marker",
            text: "Marker",
            onClick: () => {
                dispatch(entry_id_view_actions.create_entry_marker())
            }
        },
        {
            key: "possible_fields", 
            itemType: ContextualMenuItemType.Section,
            sectionProps: {
                topDivider: true,
                bottomDivider: true,
                title: "Custom Fields",
                items: fields_section
            }
        },
        {
            key: "issued_fields",
            itemType: ContextualMenuItemType.Section,
            sectionProps: {
                topDivider: true,
                title: "Issued Fields",
                items: []
            }
        }
    ];

    return <EntryIdViewContext.Provider value={dispatch}>
        <OverlayedPage>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor="white">
                {state.current != null ? 
                    <Stack horizontal tokens={{childrenGap: 8, padding: 8}}>
                        <DatePicker
                            disabled={!state.edit_view}
                            value={dateFromUnixTime(state.current.entry.day)}
                            onSelectDate={d => {
                                dispatch(entry_id_view_actions.update_entry(unixTimeFromDate(d)))
                            }}
                        />
                        {allow_edit ?
                            <IconButton 
                                iconProps={{iconName: "Edit"}} 
                                onClick={() => dispatch(entry_id_view_actions.set_edit_view(!state.edit_view))}
                            />
                            :
                            null
                        }
                        {state.edit_view ?
                            <>
                                <DefaultButton
                                    text="Add"
                                    iconProps={{iconName: "Add"}}
                                    menuProps={{items: entry_options}}
                                />
                                <DefaultButton
                                    text="Save"
                                    primaryDisabled={!state.changes_made}
                                    split
                                    iconProps={{iconName: "Save"}}
                                    onClick={() => sendEntry()}
                                    menuProps={{
                                        items: [
                                            {
                                                key: "reset",
                                                text: "Reset",
                                                disabled: !state.changes_made,
                                                iconProps: {iconName: "Refresh"},
                                                onClick: () => dispatch(entry_id_view_actions.reset_entry())
                                            },
                                            {
                                                key: "delete",
                                                text: "Delete",
                                                iconProps: {iconName: "Delete"},
                                                onClick: () => dispatch(entry_id_view_actions.set_prep_delete(true))
                                            }
                                        ]
                                    }}
                                />
                            </>
                            :
                            null
                        }
                    </Stack>
                    :
                    state.loading ?
                        <h4>Loading</h4>
                        :
                        <h4>No Entry to Show</h4>
                }
                <IconButton 
                    iconProps={{iconName: "Cancel"}} 
                    style={{position: "absolute", top: 0, right: 0}}
                    onClick={() => {
                        let url = urlFromLocation(location);

                        if (url.searchParams.has("prev")) {
                            history.push(url.searchParams.get("prev"));
                        } else {
                            let new_path = location.pathname.split("/");
                            new_path.pop();

                            history.push(new_path.join("/"));
                        }
                    }}
                />
            </Sticky>
            <Stack tokens={{childrenGap: 8, padding: "0 8px 8px"}}>
                {state.current != null ?
                    state.edit_view ?
                        <>
                            <TagPicker
                                inputProps={{placeholder: state.current.tags.length === 0 ? "Tags" : ""}}
                                selectedItems={tags_state.loading ? [] : state.current.tags.map(v => (
                                    {key: v, name: tags_state.mapping[v].title}
                                ))}
                                componentRef={tag_picker}
                                onRenderItem={(props) => {
                                    return <TagItem {...props} styles={{
                                        root: {
                                            backgroundColor: tags_state.mapping[props.item.key].color,
                                            color: getBrightness(tags_state.mapping[props.item.key].color) < 65 ? 
                                                "white" : null
                                        },
                                        close: {
                                            backgroundColor: "rgb(243,242,241)"
                                        }
                                    }}>{props.item.name}</TagItem>
                                }}
                                onRenderSuggestionsItem={(props) => {
                                    return <TagItemSuggestion styles={{}}>{props.name}</TagItemSuggestion>
                                }}
                                onResolveSuggestions={(filter, selected) => {
                                    return arrayFilterMap(
                                        tags_state.tags, 
                                        (value) => ((filter.trim() === "?" || value.title.indexOf(filter) !== -1) && !(value.id in state.tag_mapping)),
                                        (value) => ({key: value.id, name: value.title} as ITag)
                                    );
                                }}
                                getTextFromItem={(item) => item.name}
                                onItemSelected={(item) => {
                                    if (tag_picker.current && tag_picker.current.items.some(v => v.key === item.key)) {
                                        return null;
                                    } else {
                                        return item;
                                    }
                                }}
                                onChange={(items) => {
                                    dispatch(entry_id_view_actions.set_tags(items.map(v => (v.key as number))))
                                }}
                            />
                            <TextEntryEditView text_entries={state.current.text_entries}/>
                            <EntryMarkerEditView markers={state.current.markers}/>
                            {!custom_fields_state.loading ?
                                <CustomFieldEntriesEditView 
                                    custom_fields={custom_fields_state.custom_fields} 
                                    custom_field_entries={state.current.custom_field_entries}
                                />
                                :
                                <h6>Loading</h6>
                            }
                        </>
                        :
                        <>
                            <Stack horizontal wrap tokens={{childrenGap: 4}}>
                                {!tags_state.loading ? state.current.tags.map(v => <TagToken
                                    key={v} color={tags_state.mapping[v].color} title={tags_state.mapping[v].title}
                                />) : null}
                            </Stack>
                            <TextEntryReadView text_entries={state.current.text_entries}/>
                            <EntryMarkerReadView markers={state.current.markers}/>
                            {!custom_fields_state.loading ?
                                <CustomFieldEntriesReadView 
                                    custom_fields={custom_fields_state.custom_fields}
                                    custom_field_entries={state.current.custom_field_entries}
                                />
                                :
                                <h6>Loading</h6>
                            }
                        </>
                    :
                    null
                }
            </Stack>
            <Dialog
                hidden={!state.prep_delete}
                onDismiss={() => dispatch(entry_id_view_actions.set_prep_delete(false))}
                dialogContentProps={{
                    type: DialogType.normal,
                    title: "Delete Entry",
                    subText: "Are you sure you want to delete this entry?"
                }}
            >
                <DialogFooter>
                    <DefaultButton
                        text="Yes"
                        primary
                        onClick={() => {
                            dispatch(entry_id_view_actions.set_prep_delete(false));
                            deleteEntry();
                        }}
                    />
                    <DefaultButton
                        text="No"
                        onClick={() => dispatch(entry_id_view_actions.set_prep_delete(false))}
                    />
                </DialogFooter>
            </Dialog>
        </OverlayedPage>
    </EntryIdViewContext.Provider>
}

export default EntryId;
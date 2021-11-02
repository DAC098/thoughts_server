import { ContextualMenuItemType, DatePicker, DefaultButton, Dialog, DialogFooter, DialogType, IBasePicker, IconButton, IContextualMenuItem, ITag, Stack, Sticky, StickyPositionType, TagItem, TagItemSuggestion, TagPicker } from "@fluentui/react"
import React, { useEffect, useReducer, useRef } from "react"
import { useHistory, useLocation, useParams } from "react-router-dom"
import useAppDispatch from "../../../hooks/useAppDispatch"
import useAppSelector from "../../../hooks/useAppSelector"
import { entryIdViewSlice, EntryIdViewContext, initialState, entry_id_view_actions, EntryIdViewReducer } from "./reducer"
import { entries_actions as entries_actions } from "../../../redux/slices/entries"
import arrayFilterMap from "../../../util/arrayFilterMap"
import { getBrightness } from "../../../util/colors"
import TagToken from "../../../components/tags/TagItem"
import OverlayedPage from "../../../components/OverlayedPage"
import { stringFromLocation, urlFromLocation } from "../../../util/url"
import { dateFromUnixTime, unixTimeFromDate } from "../../../util/time"
import TextEntryEditView from "./TextEntryEditView";
import TextEntryReadView from "./TextEntryReadView";
import EntryMarkerEditView from "./EntryMarkerEditView";
import EntryMarkerReadView from "./EntryMarkerReadView";
import CustomFieldEntriesEditView from "./CustomFieldEntriesEditView";
import CustomFieldEntriesReadView from "./CustomFieldEntriesReadView";
import apiv2 from "../../../apiv2"
import AudioEntryEditView from "./AudioEntryEditView"
import BlobWrapper from "../../../util/BlobWrapper"

interface EntryIdProps {}

const EntryId = ({}: EntryIdProps) => {
    const location = useLocation();
    const history = useHistory();
    const params = useParams<{entry_id: string, user_id?: string}>();
    
    const entries_state = useAppSelector(state => state.entries);
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const tags_state = useAppSelector(state => state.tags);
    const appDispatch = useAppDispatch();
    const blob_map_ref = useRef(new Map<string, BlobWrapper>());

    const allow_edit = params.user_id == null;

    const tag_picker = useRef<IBasePicker<ITag>>(null);
    let [state, dispatch] = useReducer<EntryIdViewReducer>(entryIdViewSlice.reducer, initialState(allow_edit, params));

    const fetchEntry = () => {
        if (state.loading) {
            return;
        }

        dispatch(entry_id_view_actions.set_loading(true));

        apiv2.entries.id.get({
            id: params.entry_id, 
            user_id: params.user_id
        }).then(res => {
            dispatch(entry_id_view_actions.set_entry(res.body.data));
        }).catch(console.error).then(() => {
            dispatch(entry_id_view_actions.set_loading(false));
        })
    }

    const sendEntry = () => {
        if (state.current == null)
            return;
        
        if (state.sending)
            return;

        dispatch(entry_id_view_actions.set_sending(true));

        let promise = null;
        
        if (state.current.entry.id) {
            promise = apiv2.entries.id.put({
                id: state.current.entry.id, 
                post: {
                    entry: {
                        day: state.current.entry.day
                    },
                    tags: state.current.tags,
                    markers: state.current.markers.map(v => ({
                        id: v.id === 0 ? null : v.id,
                        title: v.title,
                        comment: v.comment
                    })),
                    custom_field_entries: Object.values(
                        state.current.custom_field_entries
                        ).map(v => ({
                            field: v.field,
                            value: v.value,
                            comment: v.comment
                        })),
                    text_entries: state.current.text_entries.map(v => ({
                        id: v.id === 0 ? null: v.id, 
                        thought: v.thought, 
                        private: v.private
                    }))
                }
            }).then(res => {
                dispatch(entryIdViewSlice.actions.set_entry(res.body.data));
                appDispatch(entries_actions.update_entry(res.body.data));
            })
        } else {
            promise = apiv2.entries.post({
                post: {
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
                }
            }).then(res => {
                let base_path = location.pathname.split("/");
                base_path.pop();
                base_path.push(res.body.data.entry.id.toString());

                history.push(stringFromLocation({
                    ...location, 
                    pathname: base_path.join("/")
                }));
                dispatch(entry_id_view_actions.set_entry(res.body.data));
                appDispatch(entries_actions.add_entry(res.body.data));
            });
        }

        promise.catch(console.error).then(() => {
            dispatch(entry_id_view_actions.set_sending(false));
        });
    }

    const deleteEntry = () => {
        if (state.current == null || state.current.entry.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch(entry_id_view_actions.set_deleting(true));

        apiv2.entries.id.del({id: state.current.entry.id}).then(() => {
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

        if (entry_id === 0) {
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

    let fields_section: IContextualMenuItem[] = [];
    let issued_fields_section: IContextualMenuItem[] = [];

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
                    dispatch(entry_id_view_actions.create_custom_field_entry(field.id.toString()))
                }
            });
        } else {
            fields_section.push({
                key: custom_fields_state.mapping[field.id].name,
                text: custom_fields_state.mapping[field.id].name,
                title: custom_fields_state.mapping[field.id].comment,
                onClick: () => {
                    dispatch(entry_id_view_actions.create_custom_field_entry(field.id.toString()))
                }
            });
        }
    }

    if (fields_section.length > 0) {
        fields_section.unshift({
            key: "add_all_fields",
            text: "Add All",
            iconProps: {
                iconName: "Add"
            },
            onClick: () => {
                dispatch(entry_id_view_actions.add_personal_custom_fields());
            }
        });
    }

    if (issued_fields_section.length > 0) {
        issued_fields_section.unshift({
            key: "add_all_fields",
            text: "Add All",
            iconProps: {
                iconName: "Add"
            },
            onClick: () => {
                dispatch(entry_id_view_actions.add_issued_by_custom_fields());
            }
        })
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
            key: "new-audio-entry",
            text: "Audio Entry",
            onClick: () => {
                dispatch(entry_id_view_actions.create_audio_entry());
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
            key: "add_all_fields",
            disabled: fields_section.length == 0 && issued_fields_section.length == 0,
            text: "Add All Fields",
            iconProps: {
                iconName: "Add"
            },
            onClick: () => {
                dispatch(entry_id_view_actions.add_all_custom_fields());
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
                            <AudioEntryEditView 
                                audio_entries={state.audio_entries} 
                                recording_in_progress={state.recording}
                            />
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
import { DatePicker, DefaultButton, Dialog, DialogFooter, DialogType, IBasePicker, IconButton, IContextualMenuItem, ITag, Label, ScrollablePane, Separator, Stack, Sticky, StickyPositionType, TagItem, TagItemSuggestion, TagPicker, Text, TextField, Toggle } from "@fluentui/react"
import React, { useContext, useEffect, useReducer, useRef } from "react"
import { useHistory, useLocation, useParams } from "react-router-dom"
import api from "../../../api"
import { MoodEntryType } from "../../../api/mood_entry_types"
import { EntryJson, MoodEntryJson, MoodFieldJson, TagJson, TextEntryJson } from "../../../api/types"
import { MoodEntryTypeEditView, MoodEntryTypeReadView } from "../../../components/mood_entries"
import { useAppDispatch, useAppSelector } from "../../../hooks/useApp"
import { entryIdViewSlice, EntryIdViewContext, initialState, TextEntryUI, entry_id_view_actions, EntryIdViewReducer } from "./reducer"
import { actions as entries_actions } from "../../../redux/slices/entries"
import arrayFilterMap from "../../../util/arrayFilterMap"
import { getBrightness } from "../../../util/colors"
import TagToken from "../../../components/tags/TagItem"

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
    text_entries: TextEntryJson[]
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

interface MoodEntryInputProps {
    field: MoodFieldJson
    entry: MoodEntryJson

    onDelete?: () => void
    onChange?: (entry: {comment: string, value: MoodEntryType}) => void
}

const MoodEntryInputs = ({
    field,
    entry,
    onDelete,
    onChange
}: MoodEntryInputProps) => {
    let similar_types = entry.value.type === field.config.type;

    return <Stack tokens={{childrenGap: 8}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <Label title={similar_types ? field.comment : ""}>{field.name}</Label>
            <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                onDelete?.();
            }}/>
        </Stack>
        <MoodEntryTypeEditView value={entry.value} config={similar_types ? field.config : null} onChange={value => {
            onChange?.({comment: entry.comment, value});
        }}/>
        <TextField type="text" placeholder="comment" value={entry.comment ?? ""} onChange={(e,v) => {
            onChange?.({comment: v, value: entry.value});
        }}/>
    </Stack>
}

interface MoodEntriesEditViewProps {
    mood_fields: {[id: string]: MoodFieldJson}
    mood_entries: MoodEntryJson[]
}

const MoodEntriesEditView = ({mood_fields, mood_entries}: MoodEntriesEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {mood_entries.map((mood_entry,index) =>
            <MoodEntryInputs
                key={mood_entry.field_id}
                field={mood_fields?.[mood_entry.field_id]}
                entry={mood_entry}
        
                onDelete={() => dispatch(entry_id_view_actions.delete_mood_entry(index))}
                onChange={(value) => dispatch(entry_id_view_actions.update_mood_entry({index, ...value}))}
            />
        )}
    </Stack>
}

interface MoodEntriessReadViewProps {
    mood_entries: MoodEntryJson[]
    mood_fields: {[id: string]: MoodFieldJson}
}

const MoodEntriesReadView = ({mood_fields, mood_entries}: MoodEntriessReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {mood_entries.map((mood_entry, index) => 
            <Stack key={mood_entry.field_id} tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <Separator alignContent="start">{mood_entry.field}</Separator>
                    <MoodEntryTypeReadView 
                        value={mood_entry.value} 
                        config={mood_fields?.[mood_entry.field_id]?.config}
                    />
                </Stack>
                <Text>{mood_entry.comment}</Text>
            </Stack>
        )}
    </Stack>
}

interface EntryIdProps {
    user_specific?: boolean
}

const EntryId = ({user_specific = false}: EntryIdProps) => {
    const location = useLocation<{entry?: EntryJson}>();
    const history = useHistory();
    const params = useParams<{entry_id: string, user_id?: string}>();
    
    const entries_state = useAppSelector(state => state.entries);
    const mood_fields_state = useAppSelector(state => state.mood_fields);
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
        
        if (state.current.id) {
            promise = api.entries.id.put(state.current.id, {
                created: state.current.created,
                tags: state.current.tags,
                mood_entries: state.current.mood_entries.map(v => {
                    return {
                        id: v.id,
                        field_id: v.field_id,
                        value: v.value,
                        comment: v.comment
                    }
                }),
                text_entries: state.current.text_entries.map(v => {
                    return {id: v.id, thought: v.thought, private: v.private}
                })
            }).then(entry => {
                dispatch(entryIdViewSlice.actions.set_entry(entry));
                appDispatch(entries_actions.update_entry(entry));
            })
        } else {
            promise = api.entries.post({
                created: state.current.created,
                tags: state.current.tags,
                mood_entries: state.current.mood_entries.map(v => {
                    return {
                        field_id: v.field_id,
                        value: v.value,
                        comment: v.comment
                    }
                }),
                text_entries: state.current.text_entries.map(v => {
                    return {thought: v.thought, private: v.private}
                })
            }).then(entry => {
                history.push(`/entries/${entry.id}`);
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

        if (state.current == null || state.current.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch(entry_id_view_actions.set_deleting(true));

        api.entries.id.del(state.current.id).then(() => {
            appDispatch(entries_actions.delete_entry(state.current.id));
            history.push("/entries");
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
                if (entry.id === entry_id) {
                    dispatch(entry_id_view_actions.set_entry(entry));
                    return;
                }
            }

            fetchEntry();
        }
    }, [params.entry_id]);

    let entry_options: IContextualMenuItem[] = [
        {key: "new-text-entry", text: "Text Entry", onClick: () => {
            dispatch(entry_id_view_actions.create_text_entry());
        }}
    ];

    for (let field_id in (mood_fields_state.mapping ?? {})) {
        if (field_id in state.existing_fields) {
            continue;
        }

        entry_options.push({
            key: mood_fields_state.mapping[field_id].name,
            text: mood_fields_state.mapping[field_id].name,
            title: mood_fields_state.mapping[field_id].comment,
            onClick: () => {
                dispatch(entry_id_view_actions.create_mood_entry(field_id))
            }
        });
    }

    console.log(state);

    return <EntryIdViewContext.Provider value={dispatch}>
        <Stack 
            horizontal
            verticalAlign="center"
            horizontalAlign="center"
            style={{
                width: "100%", height: "100%",
                backgroundColor: "rgba(0,0,0,0.5)",
                position: "absolute",
                top: 0,
                zIndex: 1
            }}
        >
            <Stack 
                style={{
                    width: 600, height: "100%",
                    backgroundColor: "white",
                    position: "relative"
                }}
            >
                <ScrollablePane>
                    <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor="white">
                        {state.current != null ? 
                            <Stack horizontal tokens={{childrenGap: 8, padding: 8}}>
                                <DatePicker
                                    disabled={!state.edit_view}
                                    value={new Date(state.current.created)}
                                    onSelectDate={d => {
                                        dispatch(entry_id_view_actions.update_entry(d.toISOString()))
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
                                let new_path = location.pathname.split("/");
                                new_path.pop();

                                history.push(new_path.join("/"));
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
                                    {!mood_fields_state.loading ?
                                        <MoodEntriesEditView 
                                            mood_fields={mood_fields_state.mapping} 
                                            mood_entries={state.current.mood_entries}
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
                                    {!mood_fields_state.loading ?
                                        <MoodEntriesReadView 
                                            mood_fields={mood_fields_state.mapping}
                                            mood_entries={state.current.mood_entries}
                                        />
                                        :
                                        <h6>Loading</h6>
                                    }
                                </>
                            :
                            null
                        }
                    </Stack>
                </ScrollablePane>
            </Stack>
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
    </EntryIdViewContext.Provider>
}

export default EntryId;
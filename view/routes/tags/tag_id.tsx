import { ColorPicker, DefaultButton, Dialog, DialogFooter, DialogType, IconButton, ScrollablePane, Stack, Sticky, StickyPositionType, SwatchColorPicker, TextField, Toggle } from "@fluentui/react"
import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import React, { Reducer, useEffect, useReducer } from "react"
import { useHistory, useParams } from "react-router"
import api from "../../apiv2"
import { cloneTag, newTag, Tag } from "../../apiv2/types"
import OverlayedPage from "../../components/OverlayedPage"
import useAppDispatch from "../../hooks/useAppDispatch"
import useAppSelector from "../../hooks/useAppSelector"
import { tags_actions } from "../../redux/slices/tags"
import { SliceActionTypes } from "../../redux/types"
import { cloneString } from "../../util/clone"

const color_swatches = {
    "#ffb900": {label: "Yellow"},
    "#ea4300": {label: "Orange"},
    "#d13438": {label: "Red"},
    "#b4009e": {label: "Magenta"},
    "#5c2d91": {label: "Purple"},
    "#0078d4": {label: "Blue"},
    "#008272": {label: "Teal"},
    "#107c10": {label: "Green"},

    "#d29200": {label: "Dark Yellow"},
    "#d83b01": {label: "Dark Orange"},
    "#a4262c": {label: "Dark Red"},
    "#5c005c": {label: "Dark Magenta"},
    "#32145a": {label: "Dark Purple"},
    "#002050": {label: "Dark Blue"},
    "#004b50": {label: "Dark Teal"},
    "#004b1c": {label: "Dark Green"},

    "#fff100": {label: "Light Yellow"},
    "#ff8c00": {label: "Light Orange"},
    //"#d13438": {label: "Red"},
    "#e3008c": {label: "Light Magenta"},
    "#b4a0ff": {label: "Light Purple"},
    "#00bcf2": {label: "Light Blue"},
    "#00B294": {label: "Light Teal"},
    "#bad80a": {label: "Light Green"},
    "#ffffff": {label: "White"},
};

const color_swatches_list = [
    {color: "#ffb900", id: "#ffb900", label: "Yellow"},
    {color: "#ea4300", id: "#ea4300", label: "Orange"},
    {color: "#d13438", id: "#d13438", label: "Red"},
    {color: "#b4009e", id: "#b4009e", label: "Magenta"},
    {color: "#5c2d91", id: "#5c2d91", label: "Purple"},
    {color: "#0078d4", id: "#0078d4", label: "Blue"},
    {color: "#008272", id: "#008272", label: "Teal"},
    {color: "#107c10", id: "#107c10", label: "Green"},

    {color: "#d29200", id: "#d29200", label: "Dark Yellow"},
    {color: "#d83b01", id: "#d83b01", label: "Dark Orange"},
    {color: "#a4262c", id: "#a4262c", label: "Dark Red"},
    {color: "#5c005c", id: "#5c005c", label: "Dark Magenta"},
    {color: "#32145a", id: "#32145a", label: "Dark Purple"},
    {color: "#002050", id: "#002050", label: "Dark Blue"},
    {color: "#004b50", id: "#004b50", label: "Dark Teal"},
    {color: "#004b1c", id: "#004b1c", label: "Dark Green"},

    {color: "#fff100", id: "#fff100", label: "Light Yellow"},
    {color: "#ff8c00", id: "#ff8c00", label: "Light Orange"},
    //{color: "#d13438", id: "#d13438", label: "Red"},
    {color: "#e3008c", id: "#e3008c", label: "Light Magenta"},
    {color: "#b4a0ff", id: "#b4a0ff", label: "Light Purple"},
    {color: "#00bcf2", id: "#00bcf2", label: "Light Blue"},
    {color: "#00B294", id: "#00B294", label: "Light Teal"},
    {color: "#bad80a", id: "#bad80a", label: "Light Green"},
    {color: "#ffffff", id: "#ffffff", label: "White"}
];

interface TagsIDViewState {
    original: Tag,
    current: Tag,

    loading: boolean,
    sending: boolean,
    deleting: boolean,

    prep_delete: boolean,
    changes_made: boolean,
    custom_color: boolean
}

function makeInitialState(): TagsIDViewState {
    return {
        original: newTag(),
        current: newTag(),

        loading: false,
        sending: false,
        deleting: false,

        prep_delete: false,
        changes_made: false,
        custom_color: false
    }
}

const tagsIDViewSlice = createSlice({
    name: "tags_id_view",
    initialState: makeInitialState(),
    reducers: {
        new_tag: (state) => {
            state.original = newTag();
            state.current = newTag();
            state.changes_made = false;
        },
        set_tag: (state, action: PayloadAction<Tag>) => {
            state.original = action.payload;
            state.current = cloneTag(action.payload);
            state.changes_made = false;
            state.custom_color = !(action.payload.color in color_swatches);
        },
        update_tag: (state, action: PayloadAction<Tag>) => {
            state.current = action.payload;
            state.changes_made = true;
        },
        reset_tag: (state) => {
            state.current = cloneTag(state.original);
            state.changes_made = false;
            state.custom_color = !(state.original.color in color_swatches);
        },

        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_sending: (state, action: PayloadAction<boolean>) => {
            state.sending = action.payload;
        },
        set_deleting: (state, action: PayloadAction<boolean>) => {
            state.deleting = action.payload;
        },
        set_prep_delete: (state, action: PayloadAction<boolean>) => {
            state.prep_delete = action.payload;
        },
        set_custom_color: (state, action: PayloadAction<boolean>) => {
            state.custom_color = action.payload;

            if (!(state.current.color in color_swatches)) {
                state.current.color = cloneString(state.original.color);
            }
        }
    }
});

const reducer_actions = {
    ...tagsIDViewSlice.actions
};

type TagsIDViewActions = SliceActionTypes<typeof reducer_actions>;
type TagsIDViewReducer = Reducer<TagsIDViewState, TagsIDViewActions>;

interface TagsIDViewProps {
}

const TagsIDView = ({}: TagsIDViewProps) => {
    const params = useParams<{tag_id: string}>();
    const history = useHistory();
    const tags_state = useAppSelector(state => state.tags);
    const app_dispatch = useAppDispatch();

    let [state, dispatch] = useReducer<TagsIDViewReducer>(
        tagsIDViewSlice.reducer, makeInitialState()
    );

    const fetchTag = () => {
        if (state.loading) {
            return;
        }

        dispatch(reducer_actions.set_loading(true));

        api.tags.id.get({id: params.tag_id}).then(res => {
            dispatch(reducer_actions.set_tag(res.body.data));
        }).catch(console.error).then(() => {
            dispatch(reducer_actions.set_loading(false))
        });
    }

    const sendTag = () => {
        if (state.sending) {
            return;
        }

        dispatch(reducer_actions.set_sending(true));

        let promise = null;

        if (state.current.id) {
            promise = api.tags.id.put({
                id: state.current.id,
                post: {
                    title: state.current.title,
                    color: state.current.color,
                    comment: state.current.comment?.length !== 0 ? state.current.comment : null ?? null
                }
            }).then(res => {
                let tag = res.body.data;
                app_dispatch(tags_actions.update_tag(tag));
                dispatch(reducer_actions.set_tag(tag));
            })
        } else {
            promise = api.tags.post({
                post: {
                    title: state.current.title,
                    color: state.current.color,
                    comment: state.current.comment?.length !== 0 ? state.current.comment : null ?? null
                }
            }).then(res => {
                let tag = res.body.data;
                app_dispatch(tags_actions.add_tag(tag));
                history.push(`/tags/${tag.id}`);
            })
        }

        promise.catch(console.error).then(() => {
            dispatch(reducer_actions.set_sending(false));
        });
    }

    const deleteTag = () => {
        if (state.current.id === 0) {
            return;
        }

        if (state.deleting) {
            return;
        }

        dispatch(reducer_actions.set_deleting(true));

        api.tags.id.del({id: state.current.id}).then(() => {
            app_dispatch(tags_actions.delete_tag(state.current.id));
            history.push("/tags");
        }).catch(e => {
            console.error(e);
            dispatch(reducer_actions.set_deleting(false));
        });
    }

    useEffect(() => {
        let tag_id = parseInt(params.tag_id);

        if (isNaN(tag_id) || tag_id === 0) {
            dispatch(reducer_actions.new_tag());
            return;
        }

        if (tags_state.loading) {
            fetchTag();
        } else {
            for (let tag of tags_state.tags) {
                if (tag.id === tag_id) {
                    dispatch(reducer_actions.set_tag(tag));
                    return;
                }
            }

            fetchTag();
        }
    }, [params.tag_id]);

    return <>
        <OverlayedPage width={450}>
            <Sticky stickyPosition={StickyPositionType.Header} stickyBackgroundColor="white">
                <Stack horizontal tokens={{padding: 8, childrenGap: 8}}>
                    <TextField
                        placeholder="Title"
                        value={state.current.title}
                        onChange={(e,v) => {
                            dispatch(reducer_actions.update_tag({
                                ...state.current,
                                title: v
                            }));
                        }}
                    />
                    <DefaultButton
                        text="Save"
                        primaryDisabled={state.sending}
                        split
                        iconProps={{iconName: "Save"}}
                        onClick={sendTag}
                        menuProps={{
                            items: [
                                {
                                    key: "reset",
                                    text: "Reset",
                                    iconProps: {iconName: "Refresh"},
                                    onClick: () => dispatch(reducer_actions.reset_tag())
                                },
                                {
                                    key: "delete",
                                    text: "Delete",
                                    iconProps: {iconName: "Delete"},
                                    onClick: () => dispatch(reducer_actions.set_prep_delete(true))
                                }
                            ]
                        }}
                    />
                    <IconButton 
                        iconProps={{iconName: "Cancel"}} 
                        style={{position: "absolute", top: 0, right: 0}}
                        onClick={() => {
                            let new_path = location.pathname.split("/");
                            new_path.pop();

                            history.push(new_path.join("/"));
                        }}
                    />
                </Stack>
            </Sticky>
            <Stack tokens={{padding: "0 8px", childrenGap: 8}}>
                <TextField
                    label="Comment"
                    type="text"
                    value={state.current.comment ?? ""}
                    onChange={(e,v) => {
                        dispatch(reducer_actions.update_tag({
                            ...state.current,
                            comment: v
                        }));
                    }}
                />
                <Toggle label="Custom Color" checked={state.custom_color} onChange={(e,c) => {
                    dispatch(reducer_actions.set_custom_color(c));
                }}/>
                {state.custom_color ?
                    <ColorPicker 
                        color={state.current.color} 
                        showPreview 
                        alphaType="none"
                        onChange={(e,c) => {
                            dispatch(reducer_actions.update_tag({
                                ...state.current,
                                color: c.str
                            }))
                        }}
                    />
                    :
                    <SwatchColorPicker
                        selectedId={state.current.color}
                        columnCount={8}
                        colorCells={color_swatches_list}
                        onChange={(e, id, color) => {
                            dispatch(reducer_actions.update_tag({
                                ...state.current,
                                color
                            }))
                        }}
                    />
                }
            </Stack>
        </OverlayedPage>
        <Dialog
            hidden={!state.prep_delete}
            onDismiss={() => dispatch(reducer_actions.set_prep_delete(false))}
            dialogContentProps={{
                type: DialogType.normal,
                title: "Delete Tag",
                subText: "Are you sure you want to delete this tag?"
            }}
        >
            <DialogFooter>
                <DefaultButton
                    text="Yes"
                    primary
                    onClick={() => {
                        dispatch(reducer_actions.set_prep_delete(false));
                        deleteTag();
                    }}
                />
                <DefaultButton
                    text="No"
                    onClick={() => dispatch(reducer_actions.set_prep_delete(false))}
                />
            </DialogFooter>
        </Dialog>
    </>
}

export default TagsIDView;
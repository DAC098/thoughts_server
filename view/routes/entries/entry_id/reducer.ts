import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { createContext, Dispatch, Reducer } from "react"
import { CustomFieldEntryType } from "../../../api/custom_field_entry_types";
import { cloneComposedEntry, ComposedEntry, makeComposedEntry, makeCustomFieldEntryJson, makeTextEntry, CustomFieldEntry, TextEntry, EntryMarker, makeEntryMarker } from "../../../api/types"
import { store } from "../../../redux/store";
import { SliceActionTypes } from "../../../redux/types";
import { cloneInteger } from "../../../util/clone";
import { unixTimeFromDate } from "../../../util/time";

interface UIKey {
    key?: number | string
}

export interface TextEntryUI extends UIKey, TextEntry {}

export interface EntryMarkerUI extends UIKey, EntryMarker {}

export interface CustomFieldEntryUI extends UIKey, CustomFieldEntry {
    error_msg?: string
}

export interface EntryUIState extends ComposedEntry {
    markers: EntryMarkerUI[],
    text_entries: TextEntryUI[]
    custom_field_entries: {[id: string]: CustomFieldEntryUI}
}

export interface EntryIdViewState {
    original?: EntryUIState
    current?: EntryUIState
    tag_mapping: {[id: string]: boolean}
    existing_fields: {[id: string]: boolean}
    changes_made: boolean
    prep_delete: boolean
    edit_view: boolean

    loading: boolean
    sending: boolean
    deleting: boolean

    invalid: boolean
}

export function initialState(allow_edit: boolean, params: {entry_id: string, user_id?: string}): EntryIdViewState {
    return {
        current: null, original: null,
        tag_mapping: {},
        existing_fields: {},
        changes_made: false,
        prep_delete: false,
        edit_view: allow_edit && params.entry_id === "0",
        invalid: false,
        loading: false, sending: false, deleting: false,
    }
}

export const entryIdViewSlice = createSlice({
    name: "entry_id_view",
    initialState: initialState(false, {entry_id: "0"}),
    reducers: {
        set_entry: (state, action: PayloadAction<ComposedEntry>) => {
            state.original = action.payload;
            state.current = cloneComposedEntry(action.payload);
            state.existing_fields = {};
            state.tag_mapping = {};
            state.changes_made = false;

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },
        reset_entry: (state) => {
            state.current = cloneComposedEntry(state.original);
            state.existing_fields = {};
            state.tag_mapping = {};
            state.changes_made = false;

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },
        new_entry: (state) => {
            const store_state = store.getState();
            let today = new Date();
            today.setHours(0);
            today.setMinutes(0);
            today.setSeconds(0);
            today.setMilliseconds(0);

            state.original = makeComposedEntry();
            state.original.entry.day = unixTimeFromDate(today);
            state.current = makeComposedEntry();
            state.current.entry.day = cloneInteger(state.original.entry.day);
            state.changes_made = true;
            state.existing_fields = {};

            for (let field of store_state.custom_fields.custom_fields) {
                let custom_field_entry = makeCustomFieldEntryJson(field.config.type);
                custom_field_entry.field = field.id;
                state.current.custom_field_entries[field.id] = custom_field_entry;
            }
        },
        update_entry: (state, action: PayloadAction<number>) => {
            state.current.entry.day = action.payload;
            state.changes_made = true;
        },
        
        create_mood_entry: (state, action: PayloadAction<string>) => {
            const store_state = store.getState();

            let field = store_state.custom_fields.mapping[action.payload];

            if (field == null) {
                return;
            }

            if (field.id in state.existing_fields) {
                return;
            }

            let custom_field_entry = makeCustomFieldEntryJson(field.config.type);
            custom_field_entry.field = field.id;

            state.current.custom_field_entries[field.id] = custom_field_entry;
            state.changes_made = true;
        },
        update_mood_entry: (state, action: PayloadAction<{index: number, comment: string, value: CustomFieldEntryType}>) => {
            state.current.custom_field_entries[action.payload.index].comment = action.payload.comment;
            state.current.custom_field_entries[action.payload.index].value = action.payload.value;
            state.changes_made = true;
        },
        delete_mood_entry: (state, action: PayloadAction<string>) => {
            delete state.current.custom_field_entries[action.payload];
            state.changes_made = true;
        },

        create_text_entry: (state) => {
            let text_entry: TextEntryUI = makeTextEntry();
            text_entry.key = Date.now().toString();

            state.current.text_entries.push(text_entry);
            state.changes_made = true;
        },
        update_text_entry: (state, action: PayloadAction<{index: number, thought: string, private: boolean}>) => {
            state.current.text_entries[action.payload.index].thought = action.payload.thought;
            state.current.text_entries[action.payload.index].private = action.payload.private;
            state.changes_made = true;
        },
        delete_text_entry: (state, action: PayloadAction<number>) => {
            state.current.text_entries.splice(action.payload, 1);
            state.changes_made = true;
        },

        create_entry_marker: (state) => {
            let entry_marker: EntryMarkerUI = makeEntryMarker();
            entry_marker.key = Date.now().toString();

            state.current.markers.push(entry_marker);
            state.changes_made = true;
        },
        update_entry_marker: (state, action: PayloadAction<{index: number, title: string, comment: string}>) => {
            state.current.markers[action.payload.index].title = action.payload.title;
            state.current.markers[action.payload.index].comment = action.payload.comment != null && action.payload.comment.length > 0 ? 
                action.payload.comment : null;
            state.changes_made = true;
        },
        delete_entry_marker: (state, action: PayloadAction<number>) => {
            state.current.markers.splice(action.payload, 1);
            state.changes_made = true;
        },

        set_tags: (state, action: PayloadAction<number[]>) => {
            state.changes_made = true;
            state.tag_mapping = {};
            state.current.tags = action.payload;

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },

        set_prep_delete: (state, action: PayloadAction<boolean>) => {
            state.prep_delete = action.payload;
        },
        set_edit_view: (state, action: PayloadAction<boolean>) => {
            state.edit_view = action.payload;
        },

        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_sending: (state, action: PayloadAction<boolean>) => {
            state.sending = action.payload;
        },
        set_deleting: (state, action: PayloadAction<boolean>) => {
            state.deleting = action.payload;
        }
    }
});

export const entry_id_view_actions = {
    ...entryIdViewSlice.actions
};

export type EntryIdViewActionsTypes = SliceActionTypes<typeof entry_id_view_actions>;
export type EntryIdViewReducer = Reducer<EntryIdViewState, EntryIdViewActionsTypes>;

export const EntryIdViewContext = createContext<Dispatch<EntryIdViewActionsTypes>>(null);
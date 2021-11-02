import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import { createContext, Dispatch, Reducer } from "react"
import apiv2 from "../../../apiv2";
import { CustomFieldEntryType } from "../../../apiv2/custom_field_entry_types";
import { cloneComposedEntry, ComposedEntry, newComposedEntry, newCustomFieldEntry, newTextEntry, CustomFieldEntry, TextEntry, EntryMarker, newEntryMarker, AudioEntry, newAudioEntry } from "../../../apiv2/types"
import RequestError from "../../../error/RequestError";
import { store } from "../../../redux/store";
import { SliceActionTypes } from "../../../redux/types";
import { cloneInteger } from "../../../util/clone";
import { unixNow, unixTimeFromDate } from "../../../util/time";

interface UIKey {
    key?: number | string
}

export interface TextEntryUI extends UIKey, TextEntry {}

export interface EntryMarkerUI extends UIKey, EntryMarker {}

export interface AudioEntryUI extends UIKey, AudioEntry {}

export interface CustomFieldEntryUI extends UIKey, CustomFieldEntry {
    error_msg?: string
}

export interface ComposedEntryUI extends ComposedEntry {
    markers: EntryMarkerUI[]
    text_entries: TextEntryUI[]
    custom_field_entries: {[id: string]: CustomFieldEntryUI}
}

export interface AudioEntryState {
    data: AudioEntryUI
}

export interface EntryIdViewState {
    original?: ComposedEntryUI
    current?: ComposedEntryUI
    tag_mapping: {[id: string]: boolean}

    audio_entries: AudioEntryState[]

    changes_made: boolean
    prep_delete: boolean
    edit_view: boolean

    loading: boolean
    sending: boolean
    deleting: boolean
    recording: boolean

    invalid: boolean
}

export function initialState(allow_edit: boolean, params: {entry_id: string, user_id?: string}): EntryIdViewState {
    return {
        current: null, original: null,
        tag_mapping: {},

        audio_entries: [],

        changes_made: false,
        prep_delete: false,
        edit_view: allow_edit && params.entry_id === "0",
        invalid: false,
        loading: false, sending: false, deleting: false,
        recording: false,
    }
}

export const entryIdViewSlice = createSlice({
    name: "entry_id_view",
    initialState: initialState(false, {entry_id: "0"}),
    reducers: {
        set_entry: (state, action: PayloadAction<ComposedEntry>) => {
            state.original = action.payload;
            state.current = cloneComposedEntry(action.payload);
            state.tag_mapping = {};
            state.changes_made = false;

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },
        reset_entry: (state) => {
            state.current = cloneComposedEntry(state.original);
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

            state.original = newComposedEntry();
            state.original.entry.day = unixTimeFromDate(today);
            state.current = newComposedEntry();
            state.current.entry.day = cloneInteger(state.original.entry.day);
            state.changes_made = true;

            for (let field of store_state.custom_fields.custom_fields) {
                let custom_field_entry = newCustomFieldEntry(field.config.type);
                custom_field_entry.field = field.id;
                state.current.custom_field_entries[field.id] = custom_field_entry;
            }
        },
        update_entry: (state, action: PayloadAction<number>) => {
            state.current.entry.day = action.payload;
            state.changes_made = true;
        },
        
        create_custom_field_entry: (state, action: PayloadAction<string>) => {
            const store_state = store.getState();

            let field = store_state.custom_fields.mapping[action.payload];

            if (field == null || field.id in state.current.custom_field_entries) {
                return;
            }

            let custom_field_entry = newCustomFieldEntry(field.config.type);
            custom_field_entry.field = field.id;

            state.current.custom_field_entries[field.id] = custom_field_entry;
            state.changes_made = true;
        },
        update_custom_field_entry: (state, action: PayloadAction<{index: number, comment: string, value: CustomFieldEntryType}>) => {
            state.current.custom_field_entries[action.payload.index].comment = action.payload.comment;
            state.current.custom_field_entries[action.payload.index].value = action.payload.value;
            state.changes_made = true;
        },
        delete_custom_field_entry: (state, action: PayloadAction<string>) => {
            delete state.current.custom_field_entries[action.payload];
            state.changes_made = true;
        },

        add_all_custom_fields: (state) => {
            const store_state = store.getState();

            for (let field of store_state.custom_fields.custom_fields) {
                if (field.id in state.current.custom_field_entries) {
                    continue;
                }

                let custom_field_entry = newCustomFieldEntry(field.config.type);
                custom_field_entry.field = field.id;

                state.current.custom_field_entries[field.id] = custom_field_entry;
                state.changes_made = true;
            }
        },

        add_personal_custom_fields: (state) => {
            const store_state = store.getState();

            for (let field of store_state.custom_fields.custom_fields) {
                if (field.issued_by != null || field.id in state.current.custom_field_entries) {
                    continue;
                }

                let custom_field_entry = newCustomFieldEntry(field.config.type);
                custom_field_entry.field = field.id;

                state.current.custom_field_entries[field.id] = custom_field_entry;
                state.changes_made = true;
            }
        },

        add_issued_by_custom_fields: (state) => {
            const store_state = store.getState();

            for (let field of store_state.custom_fields.custom_fields) {
                if (field.issued_by == null || field.id in state.current.custom_field_entries) {
                    continue;
                }

                let custom_field_entry = newCustomFieldEntry(field.config.type);
                custom_field_entry.field = field.id;

                state.current.custom_field_entries[field.id] = custom_field_entry;
                state.changes_made = true;
            }
        },

        create_text_entry: (state) => {
            let text_entry: TextEntryUI = newTextEntry();
            text_entry.key = unixNow();

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

        create_audio_entry: (state) => {
            let audio_entry: AudioEntryUI = newAudioEntry();
            audio_entry.key = unixNow();

            state.audio_entries.push({
                data: audio_entry
            });
            state.changes_made = true;
        },
        update_audio_entry: (state, action: PayloadAction<{index: number, private: boolean}>) => {
            state.audio_entries[action.payload.index].data.private = action.payload.private;
            state.changes_made = true;
        },
        delete_audio_entry: (state, action: PayloadAction<number>) => {
            state.audio_entries.splice(action.payload, 1);
            state.changes_made = true;
        },

        create_entry_marker: (state) => {
            let entry_marker: EntryMarkerUI = newEntryMarker();
            entry_marker.key = unixNow();

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
        },
        set_recording: (state, action: PayloadAction<boolean>) => {
            state.recording = action.payload;
        },
    }
});

export const entry_id_view_actions = {
    ...entryIdViewSlice.actions
};

export type EntryIdViewActionsTypes = SliceActionTypes<typeof entry_id_view_actions>;
export type EntryIdViewReducer = Reducer<EntryIdViewState, EntryIdViewActionsTypes>;
export type EntryIdViewDispatch = Dispatch<EntryIdViewActionsTypes>;
export const EntryIdViewContext = createContext<EntryIdViewDispatch>(null);
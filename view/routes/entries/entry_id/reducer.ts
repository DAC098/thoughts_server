import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { createContext, Dispatch, Reducer } from "react"
import { MoodEntryType } from "../../../api/mood_entry_types";
import { cloneEntryJson, EntryJson, makeEntryJson, makeMoodEntryJson, makeTextEntry, MoodEntryJson, TextEntryJson } from "../../../api/types"
import { store } from "../../../redux/store";
import { SliceActionTypes } from "../../../redux/types";

interface UIKey {
    key?: number | string
}

export interface TextEntryUI extends UIKey, TextEntryJson {}

export interface MoodEntryUI extends UIKey, MoodEntryJson {
    error_msg?: string
}

export interface EntryUIState extends EntryJson {
    text_entries: TextEntryUI[]
    mood_entries: MoodEntryUI[]
}

export interface EntryIdViewState {
    original?: EntryUIState
    current?: EntryUIState
    tag_mapping: {[id: string]: boolean}
    existing_fields: {[id: string]: number}
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
        set_entry: (state, action: PayloadAction<EntryJson>) => {
            state.original = action.payload;
            state.current = cloneEntryJson(action.payload);
            state.existing_fields = {};
            state.tag_mapping = {};
            state.changes_made = false;

            for (let field of state.current.mood_entries) {
                state.existing_fields[field.field_id] = field.id;
            }

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },
        reset_entry: (state) => {
            state.current = cloneEntryJson(state.original);
            state.existing_fields = {};
            state.tag_mapping = {};
            state.changes_made = false;

            for (let field of state.current.mood_entries) {
                state.existing_fields[field.field_id] = field.id;
            }

            for (let tag of state.current.tags) {
                state.tag_mapping[tag] = true;
            }
        },
        new_entry: (state) => {
            state.original = makeEntryJson();
            state.original.created = (new Date()).toISOString();
            state.current = makeEntryJson();
            state.current.created = state.original.created.slice(0);
            state.changes_made = false;
            state.existing_fields = {};
        },
        update_entry: (state, action: PayloadAction<string>) => {
            state.current.created = action.payload;
            state.changes_made = true;
        },
        
        create_mood_entry: (state, action: PayloadAction<string>) => {
            const store_state = store.getState();

            let field = store_state.mood_fields.mapping[action.payload];

            if (field == null) {
                return;
            }

            if (field.id in state.existing_fields) {
                return;
            }

            let mood_entry = makeMoodEntryJson(field.config.type);
            mood_entry.field = field.name;
            mood_entry.field_id = field.id;

            state.current.mood_entries.push(mood_entry);
            state.existing_fields[field.id] = 0;
            state.changes_made = true;
        },
        update_mood_entry: (state, action: PayloadAction<{index: number, comment: string, value: MoodEntryType}>) => {
            state.current.mood_entries[action.payload.index].comment = action.payload.comment;
            state.current.mood_entries[action.payload.index].value = action.payload.value;
            state.changes_made = true;
        },
        delete_mood_entry: (state, action: PayloadAction<number>) => {
            delete state.existing_fields[state.current.mood_entries[action.payload].id];
            state.current.mood_entries.splice(action.payload, 1);
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
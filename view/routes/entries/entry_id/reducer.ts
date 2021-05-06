import { createContext, Dispatch } from "react"
import { MoodEntryType } from "../../../api/mood_entry_types";
import { MoodFieldTypeName } from "../../../api/mood_field_types";
import { cloneEntryJson, EntryJson, makeEntryJson, makeMoodEntryJson, makeTextEntry, MoodEntryJson, MoodFieldJson, TextEntryJson } from "../../../api/types"
import { store } from "../../../redux/store";
import { getCreatedDateToString } from "../../../time";

export interface TextEntryUI extends TextEntryJson {
    key?: string
}

export interface MoodEntryUI extends MoodEntryJson {
    error_msg?: string
}

export interface EntryUIState extends EntryJson {
    text_entries: TextEntryUI[]
    mood_entries: MoodEntryUI[]
}

export interface EntryState {
    original?: EntryUIState
    current?: EntryUIState
    loading: boolean
    sending: boolean
    existing_fields: {[id: string]: number}
    changes_made: boolean
    prep_delete: boolean
    deleting: boolean

    edit_view: boolean

    invalid: boolean
}

export interface CreateMoodEntryAction {
    type: "create-mood-entry-action",
    field: number | string
}

export interface UpdateMoodEntryAction {
    type: "update-mood-entry"
    index: number
    value: {comment: string, value: MoodEntryType}
}

export interface DeleteMoodEntryAction {
    type: "delete-mood-entry"
    index: number
}

export interface CreateTextEntryAction {
    type: "create-text-entry-action"
}

export interface UpdateTextEntryAction {
    type: "update-text-entry"
    index: number
    thought: string
}

export interface DeleteTextEntryAction {
    type: "delete-text-entry"
    index: number
}

export interface UpdateEntryAction {
    type: "update-entry"
    created: string
}

export interface SetEntry {
    type: "set-entry"
    entry: EntryJson
}

export interface SetLoading {
    type: "set-loading"
    value: boolean
}

export interface SetSending {
    type: "set-sending"
    value: boolean
}

export interface SetMoodFields {
    type: "set-mood-fields"
    fields: MoodFieldJson[]
}

export interface ResetEntry {
    type: "reset-entry"
}

export interface NewEntry {
    type: "new-entry"
}

export interface PrepDelete {
    type: "prep-delete",
    value: boolean
}

export interface SetDeleting {
    type: "set-deleting",
    value: boolean
}

export interface SetEditView {
    type: "set-edit",
    value: boolean
}

export type EntryStateActions = UpdateMoodEntryAction | UpdateTextEntryAction | UpdateEntryAction |
    SetEntry | SetLoading | SetSending |
    ResetEntry | NewEntry |
    SetMoodFields |
    CreateMoodEntryAction | CreateTextEntryAction |
    DeleteMoodEntryAction | DeleteTextEntryAction |
    PrepDelete | SetDeleting |
    SetEditView;

export function entryStateReducer(state: EntryState, action: EntryStateActions): EntryState {
    const store_state = store.getState();

    switch (action.type) {
        case "set-entry":{
            let original = action.entry;
            let current = cloneEntryJson(action.entry);
            let existing_fields = {};

            for (let field of current.mood_entries) {
                existing_fields[field.field_id] = field.id;
            }

            return {
                ...state,
                original,
                current,
                existing_fields,
                changes_made: false
            };
        }
        case "set-loading": {
            return {
                ...state,
                loading: action.value
            }
        }
        case "set-sending": {
            return {
                ...state,
                sending: action.value
            }
        }
        case "reset-entry": {
            let current = cloneEntryJson(state.original);
            let existing_fields = {};

            for (let field of current.mood_entries) {
                existing_fields[field.field_id] = field.id;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: false
            }
        }
        case "new-entry": {
            let original = makeEntryJson();
            original.created = (new Date()).toISOString();
            let current = makeEntryJson();
            current.created = original.created.slice(0);

            return {
                ...state,
                original,
                current,
                existing_fields: {},
                changes_made: true
            }
        }
        case "update-entry": {
            let current = cloneEntryJson(state.current);
            current.created = action.created;

            return {
                ...state,
                current,
                changes_made: true
            };
        }
        case "create-mood-entry-action": {
            let field = store_state.mood_fields.mapping[action.field];

            if (field == null) {
                console.log("field requested was not found");
                return {
                    ...state
                };
            }

            if (field.id in state.existing_fields) {
                console.log("field requested already exists");
                return {
                    ...state
                };
            }

            let current = cloneEntryJson(state.current);
            let existing_fields = {};
            let mood_entry = makeMoodEntryJson(field.config.type);
            mood_entry.field = field.name;
            mood_entry.field_id = field.id;

            current.mood_entries.push(mood_entry);

            for (let f of current.mood_entries) {
                existing_fields[f.field_id] = 0;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: true
            }
        }
        case "update-mood-entry": {
            let current = cloneEntryJson(state.current);
            current.mood_entries[action.index].comment = action.value.comment;
            current.mood_entries[action.index].value = action.value.value;

            return {
                ...state,
                current,
                changes_made: true
            };
        }
        case "delete-mood-entry": {
            let existing_fields = {};
            let current = cloneEntryJson(state.current);
            current.mood_entries.splice(action.index, 1);

            for (let f of current.mood_entries) {
                existing_fields[f.field_id] = f.id;
            }

            return {
                ...state,
                current,
                existing_fields,
                changes_made: true
            }
        }
        case "create-text-entry-action": {
            let current = cloneEntryJson(state.current);
            let text_entry: TextEntryUI = makeTextEntry();
            text_entry.key = Date.now().toString();
            current.text_entries.push(text_entry);

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "update-text-entry": {
            let current = cloneEntryJson(state.current);
            current.text_entries[action.index].thought = action.thought;

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "delete-text-entry": {
            let current = cloneEntryJson(state.current);
            current.text_entries.splice(action.index, 1);

            return {
                ...state,
                current,
                changes_made: true
            }
        }
        case "prep-delete": {
            return {
                ...state,
                prep_delete: action.value
            }
        }
        case "set-deleting": {
            return {
                ...state,
                deleting: action.value
            }
        }
        case "set-edit": {
            return {
                ...state,
                edit_view: action.value
            }
        }
        default: {
            return {
                ...state
            }
        }
    }
}

export const EntryStateContext = createContext<Dispatch<EntryStateActions>>(null);
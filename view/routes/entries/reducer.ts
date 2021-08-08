import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import { createContext, Dispatch, Reducer } from "react";
import { CustomField } from "../../api/types";
import { SliceActionTypes } from "../../redux/types";

export interface EntriesViewState {
    view_graph: boolean

    visible_fields: {[key: string]: boolean}
    selected_field: CustomField
}

export function initialState(custom_fields: CustomField[]): EntriesViewState {
    let visible_fields = {};
    let selected_field = null;

    for (let field of custom_fields) {
        if (selected_field == null) {
            selected_field = field;
        }

        visible_fields[field.name] = true;
    }

    return {
        view_graph: false,
        visible_fields,
        selected_field
    }
}

export const entriesViewSlice = createSlice({
    name: "entry_view",
    initialState: initialState([]),
    reducers: {
        set_view_graph: (state, action: PayloadAction<boolean>) => {
            state.view_graph = action.payload;
        },
        toggle_view_graph: (state) => {
            state.view_graph = !state.view_graph;
        },

        set_fields: (state, action: PayloadAction<CustomField[]>) => {
            state.visible_fields = {};
            state.selected_field = null;

            for (let field of action.payload) {
                if (state.selected_field == null) {
                    state.selected_field = field;
                }

                state.visible_fields[field.name] = true;
            }
        },
        toggle_visible_field: (state, action: PayloadAction<string>) => {
            state.visible_fields[action.payload] = !state.visible_fields[action.payload];
        },

        set_selected_field: (state, action: PayloadAction<CustomField>) => {
            state.selected_field = action.payload;
        }
    }
});

export const entries_view_actions = {
    ...entriesViewSlice.actions
}

export type EntriesViewActionTypes = SliceActionTypes<typeof entries_view_actions>;
export type EntriesViewReducer = Reducer<EntriesViewState, EntriesViewActionTypes>;

export const EntriesViewContext = createContext<Dispatch<EntriesViewActionTypes>>(null);
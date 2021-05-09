import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import * as api from "../../api"
import { EntryJson } from "../../api/types"

const fetchEntries = createAsyncThunk<EntryJson[],{owner: number | string, user_specific?: boolean}>(
    "entries/fetch_entries",
    ({owner, user_specific}) => {
        return user_specific ?
            api.users.id.entries.get(owner) :
            api.entries.get();
    }
)

export interface EntriesState {
    owner: number
    loading: boolean
    entries?: EntryJson[]
}

const initialState: EntriesState = {
    owner: 0,
    loading: false,
    entries: []
};

export const entries = createSlice({
    name: "entries",
    initialState,
    reducers: {
        clear_entries: (state) => {
            state.owner = 0;
            state.entries = [];
        },
        add_entry: (state, payload: PayloadAction<EntryJson>) => {
            let new_entry_date = new Date(payload.payload.created);
            let i = 0;

            for (; i < state.entries.length; ++i) {
                let entry_date = new Date(state.entries[i].created);

                if (new_entry_date.getTime() > entry_date.getTime()) {
                    break;
                }
            }

            state.entries.splice(i, 0, payload.payload);
        },
        update_entry: (state, action: PayloadAction<EntryJson>) => {
            for (let i = 0; i < state.entries.length; ++i) {
                if (state.entries[i].id === action.payload.id) {
                    state.entries[i] = action.payload;
                    break;
                }
            }
        },
        delete_entry: (state, action: PayloadAction<number>) => {
            let i = 0;

            for (; i < state.entries.length; ++i) {
                if (state.entries[i].id === action.payload) {
                    break;
                }
            }

            if (i !== state.entries.length) {
                state.entries.splice(i, 1);
            }
        }
    },
    extraReducers: builder => {
        builder.addCase(fetchEntries.pending, (state, {}) => {
            state.loading = true;
        }).addCase(fetchEntries.fulfilled, (state, {payload, meta}) => {
            state.loading = false;
            state.entries = payload;
            state.owner = typeof meta.arg.owner === "string" ? parseInt(meta.arg.owner) : meta.arg.owner;
        }).addCase(fetchEntries.rejected, (state, {}) => {
            state.loading = false;
        });
    }
});

export const actions = {
    ...entries.actions,
    fetchEntries
};
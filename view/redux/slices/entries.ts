import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import api from "../../api"
import { EntryJson, GetEntriesQuery } from "../../api/types"
import { compareDates } from "../../util/compare";
import { rand } from "../../util/rand";

const fetchEntries = createAsyncThunk<EntryJson[],{owner: number | string, user_specific?: boolean, query?: GetEntriesQuery}>(
    "entries/fetch_entries",
    ({owner, user_specific = false, query = {}}) => {
        return user_specific ?
            api.users.id.entries.get(owner, query) :
            api.entries.get(query);
    }
)

export interface EntriesState {
    key: number
    owner: number
    loading: boolean
    entries: EntryJson[]
    from?: number
    to?: number
}

const initialState: EntriesState = {
    key: 0,
    owner: 0,
    loading: false,
    entries: [],
    from: null,
    to: null
};

export const entries = createSlice({
    name: "entries",
    initialState,
    reducers: {
        clear_entries: (state) => {
            state.owner = 0;
            state.entries = [];
            state.from = null;
            state.to = null;
            state.key = 0;
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
            state.key = rand();
        },
        update_entry: (state, action: PayloadAction<EntryJson>) => {
            for (let i = 0; i < state.entries.length; ++i) {
                if (state.entries[i].id === action.payload.id) {
                    state.entries[i] = action.payload;
                    break;
                }
            }

            state.entries.sort((a, b) => {
                return compareDates(new Date(a.created), new Date(b.created));
            });
            state.key = rand();
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
                state.key = rand();
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
            state.from = meta.arg.query?.from?.getTime();
            state.to = meta.arg.query?.to?.getTime();
            state.key = rand();
        }).addCase(fetchEntries.rejected, (state, {}) => {
            state.loading = false;
        });
    }
});

export const actions = {
    ...entries.actions,
    fetchEntries
};
import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import api from "../../api"
import { ComposedEntry, GetEntriesQuery } from "../../api/types"
import { ResponseJSON } from "../../request";
import { compareDates, compareNumbers } from "../../util/compare";
import { rand } from "../../util/rand";

type FetchEntriesReturned = ComposedEntry[];
type FetchEntriesThunkArg = {owner: number | string, user_specific?: boolean, query?: GetEntriesQuery};

const fetchEntries = createAsyncThunk<FetchEntriesReturned, FetchEntriesThunkArg>(
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
    entries: ComposedEntry[]
    from?: number
    to?: number
    tags?: number[]
}

const initialState: EntriesState = {
    key: 0,
    owner: 0,
    loading: false,
    entries: [],
    from: null,
    to: null,
    tags: null
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
            state.tags = null;
            state.key = 0;
        },
        add_entry: (state, action: PayloadAction<ComposedEntry>) => {
            let new_entry_date = action.payload.entry.day;
            let i = 0;

            if (state.from != null) {
                if (new_entry_date < state.from) {
                    return;
                }
            }

            if (state.to != null) {
                if (new_entry_date > state.to) {
                    return;
                }
            }

            for (; i < state.entries.length; ++i) {
                if (new_entry_date > state.entries[i].entry.day) {
                    break;
                }
            }

            state.entries.splice(i, 0, action.payload);
            state.key = rand();
        },
        update_entry: (state, action: PayloadAction<ComposedEntry>) => {
            for (let i = 0; i < state.entries.length; ++i) {
                if (state.entries[i].entry.id === action.payload.entry.id) {
                    state.entries[i] = action.payload;
                    break;
                }
            }

            state.entries.sort((a, b) => {
                return -compareNumbers(a.entry.day, b.entry.day);
            });
            state.key = rand();
        },
        delete_entry: (state, action: PayloadAction<number>) => {
            let i = 0;

            for (; i < state.entries.length; ++i) {
                if (state.entries[i].entry.id === action.payload) {
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
        }).addCase(fetchEntries.rejected, (state, {payload, meta}) => {
            state.loading = false;
            state.entries = [];
            state.owner = typeof meta.arg.owner === "string" ? parseInt(meta.arg.owner) : meta.arg.owner;
            state.from = meta.arg.query?.from?.getTime();
            state.to = meta.arg.query?.to?.getTime();
            state.key = rand();
        });
    }
});

export const actions = {
    ...entries.actions,
    fetchEntries
};
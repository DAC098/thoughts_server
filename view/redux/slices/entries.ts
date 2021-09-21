import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import { ComposedEntry } from "../../apiv2/types"
import { GetEntriesArgs } from "../../apiv2/entries";
import apiv2 from "../../apiv2";
import { compareNumbers } from "../../util/compare";
import { rand } from "../../util/rand";
import RequestError from "../../error/RequestError";

type FetchEntriesReturned = ComposedEntry[];

const fetchEntries = createAsyncThunk<FetchEntriesReturned, GetEntriesArgs>(
    "entries/fetch_entries",
    async (args, thunkApi) => {
        try {
            let res = await apiv2.entries.get(args);
            
            return res.body.data;
        } catch(err) {
            if (err instanceof RequestError) {
                thunkApi.rejectWithValue(err.toJson())
            } else {
                thunkApi.rejectWithValue(err);
            }
        }
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
            state.owner = typeof meta.arg.user_id === "string" ? parseInt(meta.arg.user_id) : meta.arg.user_id;
            state.from = meta.arg.query?.from?.getTime();
            state.to = meta.arg.query?.to?.getTime();
            state.key = rand();
        }).addCase(fetchEntries.rejected, (state, {meta}) => {
            state.loading = false;
            state.entries = [];
            state.owner = typeof meta.arg.user_id === "string" ? parseInt(meta.arg.user_id) : meta.arg.user_id;
            state.from = meta.arg.query?.from?.getTime();
            state.to = meta.arg.query?.to?.getTime();
            state.key = rand();
        });
    }
});

export const entries_actions = {
    ...entries.actions,
    fetchEntries
};
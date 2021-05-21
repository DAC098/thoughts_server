import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import api from "../../api"
import { MoodFieldJson } from "../../api/types"

const fetchMoodFields = createAsyncThunk<MoodFieldJson[], {owner: number | string, user_specific?: boolean}>(
    "mood_fields/fetch_mood_fields",
    ({owner, user_specific}) => {
        return user_specific ?
            api.users.id.mood_fields.get(owner) :
            api.mood_fields.get()
    }
)

export interface MoodFieldsState {
    owner: number
    loading: boolean
    mood_fields: MoodFieldJson[]
    mapping: {[id: string]: MoodFieldJson}
}

const initialState: MoodFieldsState = {
    owner: 0,
    loading: false,
    mood_fields: [],
    mapping: {}
}

export const mood_fields = createSlice({
    name: "mood_fields",
    initialState,
    reducers: {
        clear_mood_fields: (state) => {
            state.owner = 0;
            state.mood_fields = [];
        },
        add_field: (state, action: PayloadAction<MoodFieldJson>) => {
            state.mood_fields.push(action.payload);
            state.mapping[action.payload.id] = action.payload;
        },
        update_field: (state, action: PayloadAction<MoodFieldJson>) => {
            for (let i = 0; i < state.mood_fields.length; ++i) {
                if (state.mood_fields[i].id === action.payload.id) {
                    state.mood_fields[i] = action.payload;
                    state.mapping[action.payload.id] = action.payload;
                    break;
                }
            }
        },
        delete_field: (state, action: PayloadAction<number>) => {
            let i = 0;

            for (; i < state.mood_fields.length; ++i) {
                if (state.mood_fields[i].id === action.payload) {
                    break;
                }
            }

            if (i !== state.mood_fields.length) {
                state.mood_fields.splice(i, 1);
                delete state.mapping[action.payload];
            }
        }
    },
    extraReducers: builder => {
        builder.addCase(fetchMoodFields.pending, (state) => {
            state.loading = true;
        }).addCase(fetchMoodFields.fulfilled, (state, {payload, meta}) => {
            state.loading = false;
            state.owner = typeof meta.arg.owner === "string" ? parseInt(meta.arg.owner) : meta.arg.owner;
            state.mood_fields = payload;
            let mapping = {};

            for (let field of payload) {
                mapping[field.id] = field;
            }

            state.mapping = mapping;
        }).addCase(fetchMoodFields.rejected, (state) => {
            state.loading = false;
        });
    }
});

export const actions = {
    ...mood_fields.actions,
    fetchMoodFields
};
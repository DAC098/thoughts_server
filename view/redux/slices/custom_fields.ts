import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import api from "../../api"
import { CustomFieldJson } from "../../api/types"

const fetchCustomFields = createAsyncThunk<CustomFieldJson[], {owner: number | string, user_specific?: boolean}>(
    "custom_fields/fetch_custom_fields",
    ({owner, user_specific}) => {
        return user_specific ?
            api.users.id.custom_fields.get(owner) :
            api.custom_fields.get()
    }
)

export interface CustomFieldsState {
    owner: number
    loading: boolean
    custom_fields: CustomFieldJson[]
    mapping: {[id: string]: CustomFieldJson}
}

const initialState: CustomFieldsState = {
    owner: 0,
    loading: false,
    custom_fields: [],
    mapping: {}
}

export const custom_fields = createSlice({
    name: "custom_fields",
    initialState,
    reducers: {
        clear_custom_fields: (state) => {
            state.owner = 0;
            state.custom_fields = [];
        },
        add_field: (state, action: PayloadAction<CustomFieldJson>) => {
            state.custom_fields.push(action.payload);
            state.mapping[action.payload.id] = action.payload;
        },
        update_field: (state, action: PayloadAction<CustomFieldJson>) => {
            for (let i = 0; i < state.custom_fields.length; ++i) {
                if (state.custom_fields[i].id === action.payload.id) {
                    state.custom_fields[i] = action.payload;
                    state.mapping[action.payload.id] = action.payload;
                    break;
                }
            }
        },
        delete_field: (state, action: PayloadAction<number>) => {
            let i = 0;

            for (; i < state.custom_fields.length; ++i) {
                if (state.custom_fields[i].id === action.payload) {
                    break;
                }
            }

            if (i !== state.custom_fields.length) {
                state.custom_fields.splice(i, 1);
                delete state.mapping[action.payload];
            }
        }
    },
    extraReducers: builder => {
        builder.addCase(fetchCustomFields.pending, (state) => {
            state.loading = true;
        }).addCase(fetchCustomFields.fulfilled, (state, {payload, meta}) => {
            state.loading = false;
            state.owner = typeof meta.arg.owner === "string" ? parseInt(meta.arg.owner) : meta.arg.owner;
            state.custom_fields = payload;
            let mapping = {};

            for (let field of payload) {
                mapping[field.id] = field;
            }

            state.mapping = mapping;
        }).addCase(fetchCustomFields.rejected, (state) => {
            state.loading = false;
        });
    }
});

export const custom_field_actions = {
    ...custom_fields.actions,
    fetchMoodFields: fetchCustomFields
};
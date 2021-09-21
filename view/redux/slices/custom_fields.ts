import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit";
import apiv2 from "../../apiv2";
import { CustomField } from "../../apiv2/types"
import { compareNumbers, compareStrings } from "../../util/compare";
import { GetCustomFieldsArgs } from "../../apiv2/custom_fields";
import RequestError from "../../error/RequestError";

const fetchCustomFields = createAsyncThunk<CustomField[], GetCustomFieldsArgs>(
    "custom_fields/fetch_custom_fields",
    async (args, thunkApi) => {
        try {
            let res = await apiv2.custom_fields.get(args);

            return res.body.data;
        } catch(err) {
            if (err instanceof RequestError) {
                thunkApi.rejectWithValue(err.toJson());
            } else {
                thunkApi.rejectWithValue(err)
            }
        }
    }
)

export interface CustomFieldsState {
    owner: number
    loading: boolean
    custom_fields: CustomField[]
    mapping: {[id: string]: CustomField}
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
        add_field: (state, action: PayloadAction<CustomField>) => {
            state.custom_fields.push(action.payload);
            state.mapping[action.payload.id] = action.payload;

            state.custom_fields.sort((a, b) => {
                let order_sort = compareNumbers(a.order, b.order);

                if (order_sort === 0) {
                    return compareStrings(a.name, b.name);
                } else {
                    return -order_sort;
                }
            })
        },
        update_field: (state, action: PayloadAction<CustomField>) => {
            for (let i = 0; i < state.custom_fields.length; ++i) {
                if (state.custom_fields[i].id === action.payload.id) {
                    state.custom_fields[i] = action.payload;
                    state.mapping[action.payload.id] = action.payload;
                    break;
                }
            }

            state.custom_fields.sort((a, b) => {
                let order_sort = compareNumbers(a.order, b.order);

                if (order_sort === 0) {
                    return compareStrings(a.name, b.name);
                } else {
                    return -order_sort;
                }
            })
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
            state.owner = typeof meta.arg.user_id === "string" ? parseInt(meta.arg.user_id) : meta.arg.user_id;
            state.custom_fields = payload;
            let mapping = {};

            for (let field of payload) {
                mapping[field.id] = field;
            }

            state.mapping = mapping;
        }).addCase(fetchCustomFields.rejected, (state, {meta}) => {
            state.loading = false;
            state.custom_fields = [];
            state.mapping = {};
            state.owner = typeof meta.arg.user_id === "string" ? parseInt(meta.arg.user_id) : meta.arg.user_id;
        });
    }
});

export const custom_field_actions = {
    ...custom_fields.actions,
    fetchCustomFields
};
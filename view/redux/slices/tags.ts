import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit"
import api from "../../api";
import { TagJson } from "../../api/types";

const fetchTags = createAsyncThunk<any, {owner: number | string, user_specific?: boolean}>(
    "tags/fetch_tags",
    ({owner, user_specific = false}) => {
        return user_specific ?
        api.users.id.tags.get(owner) :
        api.tags.get();
    }
);

export interface TagsState {
    owner: number
    loading: boolean
    tags: TagJson[],
    mapping: {[key: string]: TagJson}
}

const initialState: TagsState = {
    owner: 0,
    loading: false,
    tags: [],
    mapping: {}
};

export const tags = createSlice({
    name: "tags",
    initialState,
    reducers: {
        clear_tags: (state) => {
            state.owner = 0;
            state.tags = [];
            state.loading = false;
            state.mapping = {};
        },
        add_tag: (state, payload: PayloadAction<any>) => {

        },
        update_tag: (state, payload: PayloadAction<any>) => {

        },
        delete_tag: (state, payload: PayloadAction<any>) => {

        }
    },
    extraReducers: builder => {
        builder.addCase(fetchTags.pending, (state) => {
            state.loading = true;
        }).addCase(fetchTags.fulfilled, (state, {payload, meta}) => {
            state.owner = typeof meta.arg.owner === "string" ? parseInt(meta.arg.owner) : meta.arg.owner;
            state.loading = false;
            state.tags = payload;
            state.mapping = {};

            for (let tag of state.tags) {
                state.mapping[tag.id] = tag;
            }
        }).addCase(fetchTags.rejected, (state, {}) => {
            state.loading = false;
        })
    }
});

export const tags_actions = {
    ...tags.actions,
    fetchTags
}
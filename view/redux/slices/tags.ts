import { createAsyncThunk, createSlice, PayloadAction } from "@reduxjs/toolkit"
import api from "../../api";
import { Tag } from "../../api/types";
import { compareStrings } from "../../util/compare";

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
    tags: Tag[],
    mapping: {[key: string]: Tag}
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
        add_tag: (state, action: PayloadAction<Tag>) => {
            state.tags.push(action.payload);
            state.mapping[action.payload.id] = action.payload;

            state.tags.sort((a, b) => {
                return compareStrings(a.title, b.title);
            });
        },
        update_tag: (state, action: PayloadAction<Tag>) => {
            for (let i = 0; i < state.tags.length; ++i) {
                if (state.tags[i].id === action.payload.id) {
                    state.tags[i] = action.payload;
                    state.mapping[action.payload.id] = action.payload;
                    break;
                }
            }

            state.tags.sort((a, b) => {
                return compareStrings(a.title, b.title);
            });
        },
        delete_tag: (state, action: PayloadAction<number>) => {
            let i = 0; 

            for (; i < state.tags.length; ++i) {
                if (state.tags[i].id === action.payload) {
                    break;
                }
            }

            if (i !== state.tags.length) {
                state.tags.splice(i, 1);
                delete state.mapping[action.payload];
            }
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
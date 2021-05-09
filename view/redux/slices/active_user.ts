import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import { makeUserDataJson, UserDataJson } from "../../api/types"

export const active_user = createSlice({
    name: "active_user",
    initialState: {
        user: <UserDataJson>window["active_user"] ?? makeUserDataJson(),
        loading: false
    },
    reducers: {
        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_user: (state, action: PayloadAction<UserDataJson>) => {
            state.user = action.payload
        },
        clear_user: (state) => {
            state.user = makeUserDataJson();
        },
        update_info: (state, action: PayloadAction<{full_name: string, username: string, email?: string}>) => {
            state.user.username = action.payload.username;
            state.user.full_name = action.payload.full_name;
            state.user.email = action.payload.email;
        }
    }
});

export const actions = {
    ...active_user.actions
};
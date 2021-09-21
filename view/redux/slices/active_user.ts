import { createSlice, PayloadAction } from "@reduxjs/toolkit"
import { newUser, User } from "../../apiv2/types"

export const active_user = createSlice({
    name: "active_user",
    initialState: {
        user: <User>window["active_user"] ?? newUser(),
        loading: false
    },
    reducers: {
        set_loading: (state, action: PayloadAction<boolean>) => {
            state.loading = action.payload;
        },
        set_user: (state, action: PayloadAction<User>) => {
            state.user = action.payload
        },
        clear_user: (state) => {
            state.user = newUser();
        },
        update_info: (state, action: PayloadAction<{full_name: string, username: string, email?: string, email_verified: boolean}>) => {
            state.user.username = action.payload.username;
            state.user.full_name = action.payload.full_name;
            state.user.email = action.payload.email;
            state.user.email_verified = action.payload.email_verified;
        }
    }
});

export const actions = {
    ...active_user.actions
};
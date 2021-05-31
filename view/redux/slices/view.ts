import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { SliceActionTypes } from "../types";

interface ViewState {
    is_min_width: boolean
    visible: boolean
    user_specified: boolean
}

const initialState: ViewState = {
    is_min_width: false,
    visible: false,
    user_specified: false
}

export const view_slice = createSlice({
    name: "view",
    initialState,
    reducers: {
        set_visible: (state, action: PayloadAction<boolean>) => {
            state.user_specified = true;
            state.visible = action.payload;
        },

        update_visibility: (state, action: PayloadAction<boolean>) => {
            state.is_min_width = action.payload;

            if (state.user_specified) {
                return;
            }

            state.visible = !action.payload;
        }
    }
});

export const view_actions = {
    ...view_slice.actions
};
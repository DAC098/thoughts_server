import { configureStore } from "@reduxjs/toolkit"
import { active_user } from "./slices/active_user"
import { entries } from "./slices/entries"
import { custom_fields } from "./slices/custom_fields"
import { tags } from "./slices/tags";

export const store = configureStore({
    reducer: {
        active_user: active_user.reducer,
        entries: entries.reducer,
        custom_fields: custom_fields.reducer,
        tags: tags.reducer
    }
});

export type RootState = ReturnType<typeof store.getState>;
export type StateDispatch = typeof store.dispatch;
import { configureStore } from "@reduxjs/toolkit"
import { active_user } from "./active_user"
import { entries } from "./entries"
import { mood_fields } from "./mood_fields"

export const store = configureStore({
    reducer: {
        active_user: active_user.reducer,
        entries: entries.reducer,
        mood_fields: mood_fields.reducer
    }
});

export type RootState = ReturnType<typeof store.getState>;
export type StateDispatch = typeof store.dispatch;
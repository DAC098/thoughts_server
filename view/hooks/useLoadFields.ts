import { useAppDispatch, useAppSelector } from "./useApp";
import { MoodFieldsState, actions } from "../redux/slices/mood_fields"

export function useLoadFields(): [MoodFieldsState, (owner: number | string, user_specific: boolean) => void] {
    const mood_fields_state = useAppSelector(state => state.mood_fields);
    const dispatch = useAppDispatch();

    return [mood_fields_state, (owner, user_specific) => {
        if (mood_fields_state.loading) {
            return;
        }

        dispatch(actions.fetchMoodFields({
            owner, user_specific
        }));
    }]
}
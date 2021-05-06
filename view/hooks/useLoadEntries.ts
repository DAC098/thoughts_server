import { useAppDispatch, useAppSelector } from "./useApp";
import { actions, EntriesState } from "../redux/entries"

export function useLoadEntries(): [EntriesState, (owner: number | string, user_specific: boolean) => void] {
    const entries_state = useAppSelector(state => state.entries);
    const dispatch = useAppDispatch();

    return [entries_state, (owner: number | string, user_specific: boolean = false) => {
        if (entries_state.loading) {
            return;
        }

        dispatch(actions.fetchEntries({
            owner, user_specific
        }));
    }];
}
import { useAppDispatch, useAppSelector } from "./useApp";
import { actions } from "../redux/slices/entries"
import { GetEntriesQuery } from "../api/types";

export function useLoadEntries(): (owner: number | string, user_specific?: boolean, query?: GetEntriesQuery) => void {
    const entries_state = useAppSelector(state => state.entries);
    const dispatch = useAppDispatch();

    return (owner: number | string, user_specific: boolean = false, query: GetEntriesQuery = {}) => {
        if (entries_state.loading) {
            return;
        }

        dispatch(actions.fetchEntries({
            owner, user_specific, query
        }));
    };
}
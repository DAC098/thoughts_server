import useAppDispatch from "./useAppDispatch"
import useAppSelector from "./useAppSelector"
import { entries_actions } from "../redux/slices/entries"
import { GetEntriesArgs } from "../apiv2/entries";

export function useGlobalFetchEntries() {
    const entries_state = useAppSelector(state => state.entries);
    const dispatch = useAppDispatch();

    return (args: GetEntriesArgs) => {
        if (entries_state.loading) {
            return;
        }

        return dispatch(entries_actions.fetchEntries(args));
    };
}
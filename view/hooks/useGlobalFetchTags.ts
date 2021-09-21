import { GetTagsArgs } from "../apiv2/tags";
import { tags_actions } from "../redux/slices/tags";
import useAppDispatch from "./useAppDispatch";
import useAppSelector from "./useAppSelector";

export function useGlobalFetchTags() {
    const tags_state = useAppSelector(state => state.tags);
    const dispatch = useAppDispatch();

    return (args: GetTagsArgs) => {
        if (tags_state.loading) {
            return;
        }

        return dispatch(tags_actions.fetchTags(args));
    }
}
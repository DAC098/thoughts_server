import { useAppDispatch, useAppSelector } from "./useApp";
import { CustomFieldsState, custom_field_actions } from "../redux/slices/custom_fields"

export function useLoadFields(): [CustomFieldsState, (owner: number | string, user_specific: boolean) => void] {
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const dispatch = useAppDispatch();

    return [custom_fields_state, (owner, user_specific) => {
        if (custom_fields_state.loading) {
            return;
        }

        dispatch(custom_field_actions.fetchMoodFields({
            owner, user_specific
        }));
    }]
}
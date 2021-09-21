import useAppDispatch from "./useAppDispatch"
import useAppSelector from "./useAppSelector"
import { custom_field_actions } from "../redux/slices/custom_fields"
import { GetCustomFieldsArgs } from "../apiv2/custom_fields";

export function useGlobalFetchCustomFields() {
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const dispatch = useAppDispatch();

    return (args: GetCustomFieldsArgs) => {
        if (custom_fields_state.loading) {
            return;
        }

        dispatch(custom_field_actions.fetchCustomFields(args));
    };
}
import { useParams } from "react-router";
import { useAppSelector } from "./useApp";

export function useOwner(favor_param: boolean = false) {
    const params = useParams<{user_id?: string}>();
    const active_user_state = useAppSelector(state => state.active_user);

    return favor_param ? parseInt(params.user_id) : (active_user_state.user.id);
}
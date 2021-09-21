import { useParams } from "react-router";

export function useUserId() {
    const params = useParams<{user_id?: string}>();

    return params.user_id != null ? parseInt(params.user_id) : null;
}
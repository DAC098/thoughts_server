import { json } from "../request"
import { UserDataJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<UserDataJson>(`/users/${user_id}`);

    return body.data;
}

export * as entries from "./users_id_entries"
export * as mood_fields from "./users_id_mood_fields"
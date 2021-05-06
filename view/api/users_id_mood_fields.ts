import { getURL } from ".";
import { json } from "../request";
import { MoodFieldJson } from "./types";

export async function get(user: number | string) {
    let {body} = await json.get<MoodFieldJson[]>(getURL(`/users/${user}/mood_fields`));

    return body.data;
}

export * as id from "./users_id_mood_fields_id"

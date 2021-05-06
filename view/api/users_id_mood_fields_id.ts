import { getURL } from ".";
import { json } from "../request";
import { MoodFieldJson } from "./types";

export async function get(user: number | string, field: number | string) {
    let {body} = await json.get<MoodFieldJson>(getURL(`/users/${user}/mood_fields/${field}`));

    return body.data;
}
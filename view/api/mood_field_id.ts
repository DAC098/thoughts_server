import { getURL } from ".";
import { json } from "../request";
import { MoodFieldJson, PutMoodField } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<MoodFieldJson>(getURL(`/mood_fields/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutMoodField) {
    let {body} = await json.put<MoodFieldJson>(getURL(`/mood_fields/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(getURL(`/mood_fields/${id}`));
}
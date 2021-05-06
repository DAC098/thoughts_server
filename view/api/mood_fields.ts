import { getURL } from ".";
import { MoodFieldJson, PostMoodField } from "./types";
import { json } from "../request";

export async function get() {
    let {body} = await json.get<MoodFieldJson[]>(getURL("/mood_fields"));

    return body.data;
}

export async function post(post: PostMoodField) {
    let {body} = await json.post<MoodFieldJson>(getURL("/mood_fields"), post);

    return body.data;
}

export * as id from "./mood_field_id"
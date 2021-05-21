import { getURL } from ".";
import { json } from "../request";
import { TagJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<TagJson[]>(getURL(`/users/${user_id}/tags`));

    return body.data;
}
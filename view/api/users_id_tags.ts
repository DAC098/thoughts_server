import { json } from "../request";
import { urlFromString } from "../util/url";
import { TagJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<TagJson[]>(urlFromString(`/users/${user_id}/tags`));

    return body.data;
}
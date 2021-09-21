import { json } from "../request";
import { urlFromString } from "../util/url";
import { Tag } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<Tag[]>(urlFromString(`/users/${user_id}/tags`));

    return body.data;
}
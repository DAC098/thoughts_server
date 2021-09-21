import { json } from "../request";
import { urlFromString } from "../util/url";
import { Tag } from "./types";

export async function get(tag_id: number | string) {
    let {body} = await json.get<Tag>(urlFromString(`/tags/${tag_id}`));

    return body.data;
}

export async function put(tag_id: number | string, posted: any) {
    let {body} = await json.put<Tag>(urlFromString(`/tags/${tag_id}`), posted);

    return body.data;
}

export async function del(tag_id: number | string) {
    let {body} = await json.delete(urlFromString(`/tags/${tag_id}`));

    return body.data;
}
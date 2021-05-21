import { json } from "../request";
import { TagJson } from "./types";

export async function get(tag_id: number | string) {
    let {body} = await json.get<TagJson>(`/tags/${tag_id}`);

    return body.data;
}

export async function put(tag_id: number | string, posted: any) {
    let {body} = await json.put<TagJson>(`/tags/${tag_id}`, posted);

    return body.data;
}

export async function del(tag_id: number | string) {
    let {body} = await json.delete(`/tags/${tag_id}`);

    return body.data;
}
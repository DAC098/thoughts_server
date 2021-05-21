import { json } from "../request";
import { TagJson } from "./types";

export async function get() {
    let {body} = await json.get<TagJson[]>("/tags");

    return body.data;
}

export async function post(posted: any) {
    let {body} = await json.post<TagJson>("/tags", posted);

    return body.data;
}

export * as id from "./tags_id"
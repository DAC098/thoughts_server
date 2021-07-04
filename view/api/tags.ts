import { json } from "../request";
import { urlFromString } from "../util/url";
import { TagJson } from "./types";

export async function get() {
    let {body} = await json.get<TagJson[]>(urlFromString("/tags"));

    return body.data;
}

export async function post(posted: any) {
    let {body} = await json.post<TagJson>(urlFromString("/tags"), posted);

    return body.data;
}

export * as id from "./tags_id"
import { getURL } from ".";
import { CustomFieldJson, PostCustomField } from "./types";
import { json } from "../request";

export async function get() {
    let {body} = await json.get<CustomFieldJson[]>(getURL("/custom_fields"));

    return body.data;
}

export async function post(post: PostCustomField) {
    let {body} = await json.post<CustomFieldJson>(getURL("/custom_fields"), post);

    return body.data;
}

export * as id from "./custom_field_id"
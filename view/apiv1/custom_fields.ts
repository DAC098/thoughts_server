import { CustomField, PostCustomField } from "./types";
import { json } from "../request";
import { urlFromString } from "../util/url";

export async function get() {
    let {body} = await json.get<CustomField[]>(urlFromString("/custom_fields"));

    return body.data;
}

export async function post(post: PostCustomField) {
    let {body} = await json.post<CustomField>(urlFromString("/custom_fields"), post);

    return body.data;
}

export * as id from "./custom_field_id"
import { getURL } from ".";
import { json } from "../request";
import { CustomFieldJson, PutCustomField } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<CustomFieldJson>(getURL(`/custom_fields/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutCustomField) {
    let {body} = await json.put<CustomFieldJson>(getURL(`/custom_fields/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(getURL(`/custom_fields/${id}`));
}
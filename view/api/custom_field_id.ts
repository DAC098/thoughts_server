import { json } from "../request";
import { urlFromString } from "../util/url";
import { CustomFieldJson, PutCustomField } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<CustomFieldJson>(urlFromString(`/custom_fields/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutCustomField) {
    let {body} = await json.put<CustomFieldJson>(urlFromString(`/custom_fields/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(urlFromString(`/custom_fields/${id}`));
}
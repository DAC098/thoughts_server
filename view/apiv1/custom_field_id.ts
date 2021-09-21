import { json } from "../request";
import { urlFromString } from "../util/url";
import { CustomField, PutCustomField } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<CustomField>(urlFromString(`/custom_fields/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutCustomField) {
    let {body} = await json.put<CustomField>(urlFromString(`/custom_fields/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(urlFromString(`/custom_fields/${id}`));
}
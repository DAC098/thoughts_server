import { json } from "../request";
import { urlFromString } from "../util/url";
import { CustomFieldJson } from "./types";

export async function get(user: number | string, field: number | string) {
    let {body} = await json.get<CustomFieldJson>(urlFromString(`/users/${user}/custom_fields/${field}`));

    return body.data;
}
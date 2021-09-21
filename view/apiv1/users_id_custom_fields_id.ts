import { json } from "../request";
import { urlFromString } from "../util/url";
import { CustomField } from "./types";

export async function get(user: number | string, field: number | string) {
    let {body} = await json.get<CustomField>(urlFromString(`/users/${user}/custom_fields/${field}`));

    return body.data;
}
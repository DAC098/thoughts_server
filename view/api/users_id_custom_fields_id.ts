import { getURL } from ".";
import { json } from "../request";
import { CustomFieldJson } from "./types";

export async function get(user: number | string, field: number | string) {
    let {body} = await json.get<CustomFieldJson>(getURL(`/users/${user}/custom_fields/${field}`));

    return body.data;
}
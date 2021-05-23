import { getURL } from ".";
import { json } from "../request";
import { CustomFieldJson } from "./types";

export async function get(user: number | string) {
    let {body} = await json.get<CustomFieldJson[]>(getURL(`/users/${user}/custom_fields`));

    return body.data;
}

export * as id from "./users_id_custom_fields_id"

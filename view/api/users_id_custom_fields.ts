import { json } from "../request";
import { urlFromString } from "../util/url";
import { CustomField } from "./types";

export async function get(user: number | string) {
    let {body} = await json.get<CustomField[]>(urlFromString(`/users/${user}/custom_fields`));

    return body.data;
}

export * as id from "./users_id_custom_fields_id"

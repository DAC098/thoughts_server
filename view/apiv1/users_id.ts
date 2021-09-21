import { json } from "../request"
import { urlFromString } from "../util/url";
import { UserDataJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<UserDataJson>(urlFromString(`/users/${user_id}`));

    return body.data;
}

export * as entries from "./users_id_entries"
export * as custom_fields from "./users_id_custom_fields"
export * as tags from "./users_id_tags"
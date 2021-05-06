import { getURL } from ".";
import { EntryJson } from "./types";
import { json } from "../request";

export async function get(user: number | string) {
    let {body} = await json.get<EntryJson[]>(getURL(`/users/${user}/entries`));

    return body.data;
}

export * as id from "./users_id_entries_id"
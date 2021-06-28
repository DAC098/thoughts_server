import { getURL } from ".";
import { EntryJson, GetEntriesQuery } from "./types";
import { json } from "../request";

export async function get(user: number | string, query: GetEntriesQuery = {}) {
    let url = getURL(`/users/${user}/entries`);

    if (query.from != null) {
        if (typeof query.from === "string") {
            url.searchParams.append("from", query.from);
        } else {
            url.searchParams.append("from", query.from.toISOString());
        }
    }

    if (query.to != null) {
        if (typeof query.to === "string") {
            url.searchParams.append("to", query.to);
        } else {
            url.searchParams.append("to", query.to.toISOString());
        }
    }

    return json.get<EntryJson[]>(url);
}

export * as id from "./users_id_entries_id"
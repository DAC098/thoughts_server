import { ComposedEntry, GetEntriesQuery } from "./types";
import { json } from "../request";
import { unixTimeFromDate } from "../util/time";
import { urlFromString } from "../util/url";

export async function get(user: number | string, query: GetEntriesQuery = {}) {
    let url = urlFromString(`/users/${user}/entries`);

    if (query.from != null) {
        url.searchParams.append("from", unixTimeFromDate(query.from).toString());
    }

    if (query.to != null) {
        url.searchParams.append("to", unixTimeFromDate(query.to).toString());
    }

    let {body} = await json.get<ComposedEntry[]>(url);

    return body.data;
}

export * as id from "./users_id_entries_id"
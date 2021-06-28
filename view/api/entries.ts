import { getURL } from ".";
import { json } from "../request";
import { EntryJson, GetEntriesQuery, PostEntry } from "./types";

export async function get(query: GetEntriesQuery = {}) {
    let url = getURL("/entries");

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

export async function post(post: PostEntry) {
    let {body} = await json.post<EntryJson>(getURL("/entries"), post);

    return body.data;
}

export * as id from "./entries_id"
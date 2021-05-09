import { getURL } from ".";
import { json } from "../request";
import { getCreatedDateToString } from "../time"
import { EntryJson, GetEntriesQuery, PostEntry } from "./types";

export async function get(query: GetEntriesQuery = {}) {
    let url = new URL("/entries", window.location.origin);

    if (query.from != null) {
        if (typeof query.from === "string") {
            url.searchParams.append("from", query.from);
        } else {
            url.searchParams.append("from", getCreatedDateToString(query.from));
        }
    }

    if (query.to != null) {
        if (typeof query.to === "string") {
            url.searchParams.append("to", query.to);
        } else {
            url.searchParams.append("to", getCreatedDateToString(query.to));
        }
    }
    
    let {body} = await json.get<EntryJson[]>(url);

    return body.data;
}

export async function post(post: PostEntry) {
    let {body} = await json.post<EntryJson>(getURL("/entries"), post);

    return body.data;
}

export * as id from "./entries_id"
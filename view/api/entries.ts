import { json } from "../request";
import { unixTimeFromDate } from "../util/time";
import { urlFromString } from "../util/url";
import { EntryJson, GetEntriesQuery, PostEntry } from "./types";

export async function get(query: GetEntriesQuery = {}) {
    let url = urlFromString("/entries");

    if (query.from != null) {
        url.searchParams.append("from", unixTimeFromDate(query.from).toString());
    }

    if (query.to != null) {
        url.searchParams.append("to", unixTimeFromDate(query.to).toString());
    }

    let {body} = await json.get<EntryJson[]>(url);

    return body.data;
}

export async function post(post: PostEntry) {
    let {body} = await json.post<EntryJson>(urlFromString("/entries"), post);

    return body.data;
}

export * as id from "./entries_id"
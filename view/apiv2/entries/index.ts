import { json } from "../../request";
import { Vec } from "../../rust_types";
import { unixTimeStrFromDate } from "../../util/time";
import { urlFromString } from "../../util/url";
import { CustomFieldEntryType } from "../custom_field_entry_types";
import { ComposedEntry } from "../types";

export * as id from "./entry_id";

export interface GetEntriesQuery {
    from?: Date,
    to?: Date,
    tags?: number[],
    from_marker?: number,
    to_marker?: number
}

export interface GetEntriesArgs {
    query?: GetEntriesQuery,
    user_id?: number | string
}

export async function get({
    query = {},
    user_id = null
}: GetEntriesArgs) {
    let url = user_id != null ? urlFromString(`/users/${user_id}/entries`) : urlFromString("/entries");

    if (query.from != null && query.from_marker == null) {
        url.searchParams.append("from", unixTimeStrFromDate(query.from));
    }

    if (query.to != null && query.to_marker == null) {
        url.searchParams.append("to", unixTimeStrFromDate(query.to));
    }

    if (query.tags != null) {
        url.searchParams.append("tags", query.tags.join(","))
    }

    if (query.from_marker != null) {
        url.searchParams.append("from_marker", query.from_marker.toString());
    }

    if (query.to_marker != null) {
        url.searchParams.append("to_marker", query.to_marker.toString());
    }

    return await json.get<Vec<ComposedEntry>>(url);
}

export interface PostEntry {
    day: number
}

export interface PostCustomFieldEntry {
    field: number,
    value: CustomFieldEntryType,
    comment?: string
}

export interface PostEntryMarker {
    title: string
    comment?: string
}

export interface PostTextEntry {
    thought: string
    private: boolean
}

export interface PostComposedEntry {
    entry: PostEntry
    tags?: number[]
    custom_field_entries?: PostCustomFieldEntry[]
    text_entries?: PostTextEntry[]
    markers?: PostEntryMarker[]
}

export interface PostEntryArgs {
    post: PostComposedEntry
}

export async function post({post}: PostEntryArgs) {
    return await json.post<ComposedEntry>(urlFromString("/entries"), post);
}
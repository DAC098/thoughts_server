import { CustomFieldEntryType } from "../../apiv2/custom_field_entry_types";
import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { ComposedEntry } from "../types";

export interface GetEntryIdArgs {
    id: number | string
    user_id?: number | string
}

export async function get({
    id,
    user_id = null
}: GetEntryIdArgs) {
    return await json.get<ComposedEntry>(urlFromString(
        user_id == null ? `/entries/${id}` : `/users/${user_id}/entries/${id}`
    ));
}

export interface PutTextEntry {
    id?: number
    thought: string
    private: boolean
}

export interface PutCustomFieldEntry {
    field: number
    value: CustomFieldEntryType
    comment?: string
}

export interface PutEntryMarker {
    id?: number
    title: string
    comment?: string
}

export interface PutEntry {
    day: number
}

export interface PutComposedEntry {
    entry: PutEntry
    tags?: number[]
    markers?: PutEntryMarker[]
    custom_field_entries?: PutCustomFieldEntry[]
    text_entries?: PutTextEntry[]
}

export interface PutEntryIdArgs {
    id: number | string
    post: PutComposedEntry
}

export async function put({
    id, post
}: PutEntryIdArgs) {
    return await json.put<ComposedEntry>(urlFromString(`/entries/${id}`), post);
}

export interface DeleteEntryIdArgs {
    id: number | string
}

export async function del({id}: DeleteEntryIdArgs) {
    return await json.delete(urlFromString(`/entries/${id}`));
}
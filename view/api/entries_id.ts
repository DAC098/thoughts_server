import { getURL } from ".";
import { json } from "../request";
import { EntryJson, PutEntry } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<EntryJson>(getURL(`/entries/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutEntry) {
    let {body} = await json.put<EntryJson>(getURL(`/entries/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(getURL(`/entries/${id}`));
}
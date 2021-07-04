import { json } from "../request";
import { urlFromString } from "../util/url";
import { EntryJson, PutEntry } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<EntryJson>(urlFromString(`/entries/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutEntry) {
    let {body} = await json.put<EntryJson>(urlFromString(`/entries/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(urlFromString(`/entries/${id}`));
}
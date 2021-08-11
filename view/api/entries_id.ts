import { json } from "../request";
import { urlFromString } from "../util/url";
import { ComposedEntry, PutComposedEntry } from "./types";

export async function get(id: number | string) {
    let {body} = await json.get<ComposedEntry>(urlFromString(`/entries/${id}`));

    return body.data;
}

export async function put(id: number | string, put: PutComposedEntry) {
    let {body} = await json.put<ComposedEntry>(urlFromString(`/entries/${id}`), put);

    return body.data;
}

export async function del(id: number | string) {
    await json.delete(urlFromString(`/entries/${id}`));
}
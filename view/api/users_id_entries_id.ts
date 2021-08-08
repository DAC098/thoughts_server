import { json } from "../request";
import { urlFromString } from "../util/url";
import { ComposedEntry } from "./types";

export async function get(user: number | string, entry: number | string) {
    let {body} = await json.get<ComposedEntry>(urlFromString(`/users/${user}/entries/${entry}`));

    return body.data;
}
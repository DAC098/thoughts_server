import { getURL } from ".";
import { json } from "../request";
import { EntryJson } from "./types";

export async function get(user: number | string, entry: number | string) {
    let {body} = await json.get<EntryJson>(getURL(`/users/${user}/entries/${entry}`));

    return body.data;
}
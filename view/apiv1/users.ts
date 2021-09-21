import { json } from "../request";
import { urlFromString } from "../util/url";
import { UserListJson } from "./types";

export async function get() {
    let {body} = await json.get<UserListJson>(urlFromString("/users"));

    return body.data;
}

export * as id from "./users_id"
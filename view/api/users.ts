import { getURL } from ".";
import { json } from "../request";
import { UserListJson } from "./types";

export async function get() {
    let {body} = await json.get<UserListJson>(getURL("/users"));

    return body.data;
}

export * as id from "./users_id"
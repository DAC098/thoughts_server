import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { User } from "../types";

export * as id from "./user_id"

export interface GetUsesrQuery {
    level?: number
    full_name?: string
    username?: string
}

export interface GetUsersArgs {
    query?: GetUsesrQuery
}

export async function get({query = {}}: GetUsersArgs) {
    let url = urlFromString("/users");

    if (query.level) {
        url.searchParams.append("level", query.level.toString());
    }

    if (query.full_name) {
        url.searchParams.append("full_name", query.full_name);
    }

    if (query.username) {
        url.searchParams.append("username", query.username);
    }

    return await json.get<User[]>(url);
}
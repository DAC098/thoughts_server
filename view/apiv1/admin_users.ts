import { json } from "../request"
import { urlFromString } from "../util/url";
import { UserDataJson, UserInfoJson } from "./types"

interface UserSearchQuery {
    level?: number
    username?: string
    full_name?: string
}

export async function get(search: UserSearchQuery = {}) {
    let url = urlFromString("/admin/users");

    if (search.level != null) {
        url.searchParams.append("level", search.level.toString());
    }

    if (search.username != null) {
        url.searchParams.append("username", search.username);
    }

    if (search.full_name != null) {
        url.searchParams.append("full_name", search.full_name);
    }

    let {body} = await json.get<UserDataJson[]>(url);

    return body.data;
}

export async function post(post) {
    let {body} = await json.post<UserInfoJson>(urlFromString("/admin/users"), post);

    return body.data;
}

export * as id from "./admin_users_id"
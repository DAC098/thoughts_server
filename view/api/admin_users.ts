import { json } from "../request"
import { UserDataJson } from "./types"

export async function get() {
    let {body} = await json.get<UserDataJson[]>("/admin/users");

    return body.data;
}

export async function post(post) {
    let {body} = await json.post<UserDataJson>("/admin/users", post);

    return body.data;
}

export * as id from "./admin_users_id"
import { json } from "../../../request"
import { urlFromString } from "../../../util/url"
import { User, ComposedFullUser } from "../../types";

export * as id from "./users_id"

export interface GetAdminUsersQuery {
    level?: number
    username?: string
    full_name?: string
}

export interface GetAdminUsersArgs {
    query?: GetAdminUsersQuery
}

export async function get({query = {}}: GetAdminUsersArgs) {
    let url = urlFromString("/admin/users");

    if (query.level != null) {
        url.searchParams.append("level", query.level.toString());
    }

    if (query.username != null) {
        url.searchParams.append("username", query.username);
    }

    if (query.full_name != null) {
        url.searchParams.append("full_name", query.full_name);
    }

    return await json.get<User[]>(url);
}

export interface PostUserData {
    prefix?: string
    suffix?: string
    first_name: string
    last_name: string
    middle_name?: string
    dob: string
}

export interface PostUser {
    username: string
    password: string
    email?: string
    full_name?: string
    level: number
}

export interface PostStruct {
    user: PostUser,
    data: PostUserData
}

export interface PostAdminUsersArgs {
    post: PostStruct
}

export async function post({post}: PostAdminUsersArgs) {
    return await json.post<ComposedFullUser>(urlFromString("/admin/users"), post);
}
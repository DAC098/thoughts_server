import { json } from "../../../request";
import { urlFromString } from "../../../util/url";
import { ComposedFullUser } from "../../types";

export interface GetAdminUsersIdArgs {
    id: number | string
}

export async function get({id}: GetAdminUsersIdArgs) {
    return await json.get<ComposedFullUser>(urlFromString(`/admin/users/${id}`));
}

export interface PutUserData {
    prefix?: string
    suffix?: string
    first_name: string
    last_name: string
    middle_name?: string
    dob: string
}

export interface PutUser {
    username: string
    level: number
    full_name?: string
    email?: string
}

export interface Put {
    user: PutUser
    data: PutUserData
    access: number[]
}

export interface PutAdminUsersIdArgs {
    id: number | string
    post: Put
}

export async function put({id, post}: PutAdminUsersIdArgs) {
    return await json.put<ComposedFullUser>(urlFromString(`/admin/users/${id}`), post);
}
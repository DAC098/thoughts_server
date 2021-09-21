import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { User } from "../types";

export interface PostLogin {
    username: string
    password: string
}

export interface PostLoginArgs {
    post: PostLogin
}

export async function post({post}: PostLoginArgs) {
    return await json.post<User>(urlFromString(`/auth/login`), post);
}
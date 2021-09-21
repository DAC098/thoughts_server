import { json } from "../request";
import { urlFromString } from "../util/url";
import { PostLogin, UserDataJson } from "./types";

export async function post(post: PostLogin) {
    let {body} = await json.post<UserDataJson>(urlFromString("/auth/login"), post);

    return body.data;
}
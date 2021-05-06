import { getURL } from ".";
import { json } from "../request";
import { PostLogin, UserDataJson } from "./types";

export async function post(post: PostLogin) {
    let {body} = await json.post<UserDataJson>(getURL("/auth/login"), post);

    return body.data;
}
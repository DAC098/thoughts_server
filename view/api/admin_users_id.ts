import { json } from "../request";
import { UserDataJson, UserInfoJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<UserInfoJson>(`/admin/users/${user_id}`);

    return body.data;
}

export async function put(user_id: number | string, put) {
    let {body} = await json.put<UserInfoJson>(`/admin/users/${user_id}`, put);
    
    return body.data;
}
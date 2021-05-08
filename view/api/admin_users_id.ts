import { json } from "../request";
import { UserDataJson } from "./types";

export async function get(user_id: number | string) {
    let {body} = await json.get<UserDataJson>(`/admin/users/${user_id}`);

    return body.data;
}

export async function put(user_id: number | string, put) {
    let {body} = await json.put<UserDataJson>(`/admin/users/${user_id}`, put);
    
    return body.data;
}
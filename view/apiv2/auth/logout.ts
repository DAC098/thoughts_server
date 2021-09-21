import { json } from "../../request";
import { urlFromString } from "../../util/url";

export interface PostLogoutArgs {}

export async function post({}: PostLogoutArgs) {
    return await json.post(urlFromString("/auth/logout"), {});
}
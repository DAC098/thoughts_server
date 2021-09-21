import { json } from "../../request";
import { urlFromString } from "../../util/url";

export interface GetUsersIdArgs {
    id: number | string
}

export async function get({id}: GetUsersIdArgs) {
    return await json.get(urlFromString(`/users/${id}`))
}
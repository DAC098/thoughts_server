import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { CustomFieldType } from "../custom_field_types";

export interface GetCustomFieldIdArgs {
    user_id?: number | string
    id: number | string
}

export async function get({id, user_id = null}: GetCustomFieldIdArgs) {
    return await json.get(urlFromString(
        user_id == null ? `/custom_fields/${id}` : `/users/${user_id}/custom_fields/${id}`
    ));
}

export interface PutCustomFIeld {
    name: string
    config: CustomFieldType
    comment?: string
    order: number
}

export interface PutCustomFieldIdArgs {
    id: number | string
    post: PutCustomFIeld
}

export async function put({id, post}: PutCustomFieldIdArgs) {
    return await json.put(urlFromString(`/custom_fields/${id}`), post);
}

export interface DeleteCustomFieldIdArgs {
    id: number | string
}

export async function del({id}: DeleteCustomFieldIdArgs) {
    return await json.delete(urlFromString(`/custom_fields/${id}`));
}
import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { CustomFieldType } from "../custom_field_types";

export * as id from "./field_id";

export interface GetCustomFieldsArgs {
    user_id?: number | string
}

export async function get({user_id = null}: GetCustomFieldsArgs) {
    return await json.get(urlFromString(
        user_id == null ? `/custom_fields` : `/users/${user_id}/custom_fields`
    ));
}

export interface PostCustomField {
    name: string,
    config: CustomFieldType
    comment?: string
    order: number
}

export interface PostCustomFieldsArgs {
    post: PostCustomField
}

export async function post({post}: PostCustomFieldsArgs) {
    return await json.post(urlFromString(`/custom_fields`), post);
}
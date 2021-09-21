import { json } from "../../../request";
import { urlFromString } from "../../../util/url";
import { CustomFieldType } from "../../custom_field_types";
import { GlobalCustomField } from "../../types";

export interface GetGlobalCustomFieldsArgs {}

export async function get({}: GetGlobalCustomFieldsArgs) {
    return await json.get<GlobalCustomField[]>(urlFromString("/global/custom_fields"));
}

export interface PostGlobalCustomField {
    name: string
    comment?: string
    config: CustomFieldType
}

export interface PostGlobalCustomFieldsArgs {
    post: PostGlobalCustomField
}

export async function post({post}: PostGlobalCustomFieldsArgs) {
    return await json.post<GlobalCustomField>(urlFromString("/global/custom_fields"), post);
}

export * as id from "./field_id"
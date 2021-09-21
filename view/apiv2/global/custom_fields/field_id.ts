import { json } from "../../../request";
import { urlFromString } from "../../../util/url";
import { CustomFieldType } from "../../custom_field_types";
import { GlobalCustomField } from "../../types";

export interface GetGlobalCustomFieldsIdArgs {
    id: number | string
}

export async function get({id}: GetGlobalCustomFieldsIdArgs) {
    return await json.get<GlobalCustomField>(urlFromString(`/global/custom_fields/${id}`));
}

export interface PutGlobalCustomFIeld {
    name: string
    comment?: string
    config: CustomFieldType
}

export interface PutGlobalCustomFIeldsIdArgs {
    id: number | string
    post: any
}

export async function put({id, post}: PutGlobalCustomFIeldsIdArgs) {
    return await json.put<GlobalCustomField>(urlFromString(`/global/custom_fields/${id}`), post);
}

export interface DeleteGlobalCustomFieldsIdArgs {
    id: number | string
}

export async function del({id}: DeleteGlobalCustomFieldsIdArgs) {
    return await json.delete(urlFromString(`/global/custom_fields/${id}`));
}
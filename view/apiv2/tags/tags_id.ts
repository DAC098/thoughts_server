import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { Tag } from "../types";

export interface GetTagsIdArgs {
    id: string | number
}

export async function get({id}: GetTagsIdArgs) {
    return await json.get<Tag>(urlFromString(`/tags/${id}`));
}

export interface PutTagId {
    title: string
    color: string
    comment?: string
}

export interface PutTagsIdArgs {
    id: string | number
    post: PutTagId
}

export async function put({id, post}: PutTagsIdArgs) {
    return await json.put<Tag>(urlFromString(`/tags/${id}`), post);
}

export interface DeleteTagsIdArgs {
    id: string | number
}

export async function del({id}: DeleteTagsIdArgs) {
    return await json.delete(urlFromString(`/tags/${id}`));
}
import { json } from "../../request";
import { urlFromString } from "../../util/url";
import { Tag } from "../types";

export * as id from "./tags_id"

export interface GetTagsArgs {
    user_id?: string | number
}

export async function get({user_id = null}: GetTagsArgs) {
    return await json.get<Tag[]>(urlFromString(
        user_id != null ? `/users/${user_id}/tags` : `/tags`
    ));
}

export interface PostTag {
    title: string
    color: string
    comment?: string
}

export interface PostTagsArgs {
    post: PostTag
}

export async function post({post}: PostTagsArgs) {
    return await json.post<Tag>(urlFromString(`/tags`), post);
}
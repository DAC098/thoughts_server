import { optionalCloneInteger, cloneInteger, optionalCloneString, cloneString, cloneBoolean } from "../util/clone";
import { cloneMoodEntryType, makeMoodEntryType, CustomFieldEntryType } from "./custom_field_entry_types";
import { cloneCustomFieldType, makeCustomFieldType, CustomFieldType, CustomFieldTypeName } from "./custom_field_types";

export interface IssuedByJson {
    id: number
    username: string
    full_name?: string
}

export interface CustomFieldJson {
    id: number
    name: string
    config: CustomFieldType
    comment?: string
    owner: number
    order: number
    issued_by?: IssuedByJson
}

export interface CustomFieldEntryJson {
    field: number
    name: string
    value: CustomFieldEntryType
    comment?: string
}

export interface TextEntryJson {
    id: number
    thought: string
    private: boolean
}

export interface EntryJson {
    id: number
    created: string
    owner: number
    tags: number[]
    custom_field_entries: CustomFieldEntryJson[]
    text_entries: TextEntryJson[]
}

export interface TagJson {
    id: number
    title: string
    color: string
    comment?: string
    owner: number
}

export interface GetEntriesQuery {
    from?: Date
    to?: Date
}

export interface PostCustomFieldEntry {
    field: number
    value: CustomFieldEntryType,
    comment?: string
}

export interface PostTextEntry {
    thought: string
}

export interface PostEntry {
    created: string
    tags?: number[]
    custom_field_entries?: PostCustomFieldEntry[]
    text_entries?: PostTextEntry[]
}

export interface PutTextEntry {
    id?: number
    thought: string
}

export interface PutCustomFieldEntry {
    field: number
    value: CustomFieldEntryType
    comment?: string
}

export interface PutEntry {
    created: string,
    tags: number[]
    custom_field_entries?: PutCustomFieldEntry[]
    text_entries?: PutTextEntry[]
}

export interface PostCustomField {
    name: string
    config: CustomFieldType
    comment?: string
    order: number
}

export interface PutCustomField {
    name: string
    config: CustomFieldType
    comment?: string
    order: number
}

export interface UserListItemJson {
    id: number,
    username: string,
    full_name?: string,
    ability: string
}

export interface UserListJson {
    allowed: UserListItemJson[],
    given: UserListItemJson[]
}

export interface UserDataJson {
    id: number,
    username: string,
    level: number,
    full_name?: string,
    email?: string,
    email_verified: boolean
}

export interface UserAccessInfoJson {
    id: number
    username: string
    full_name?: string
    ability: string
}

export interface UserInfoJson {
    id: number
    username: string
    level: number
    full_name?: string
    email?: string
    user_access: UserAccessInfoJson[]
}

export interface PostLogin {
    username: string
    password: string
}

export function makeCustomFieldEntryJson(type: CustomFieldTypeName = CustomFieldTypeName.Integer): CustomFieldEntryJson {
    return {
        field: 0,
        name: "",
        value: makeMoodEntryType(type),
        comment: null
    }
}

export function cloneCustomFieldEntryJson(custom_field_entry: CustomFieldEntryJson): CustomFieldEntryJson {
    return {
        field: cloneInteger(custom_field_entry.field),
        name: cloneString(custom_field_entry.name),
        value: cloneMoodEntryType(custom_field_entry.value),
        comment: optionalCloneString(custom_field_entry.comment)
    };
}

export function makeTextEntry(): TextEntryJson {
    return {
        id: null,
        thought: "",
        private: false
    }
}

export function cloneTextEntry(text_entry: TextEntryJson): TextEntryJson {
    return {
        id: optionalCloneInteger(text_entry.id),
        thought: cloneString(text_entry.thought),
        private: cloneBoolean(text_entry.private)
    };
}

export function makeEntryJson(): EntryJson {
    return {
        id: null,
        created: "",
        owner: 0,
        tags: [],
        custom_field_entries: [],
        text_entries: []
    }
}

export function cloneEntryJson(entry: EntryJson) {
    let rtn: EntryJson = {
        id: optionalCloneInteger(entry.id),
        created: cloneString(entry.created),
        owner: cloneInteger(entry.owner),
        tags: [],
        custom_field_entries: [],
        text_entries: []
    };

    for (let m of (entry.custom_field_entries ?? [])) {
        rtn.custom_field_entries.push(
            cloneCustomFieldEntryJson(m)
        );
    }

    for (let t of (entry.text_entries ?? [])) {
        rtn.text_entries.push(
            cloneTextEntry(t)
        );
    }

    for (let t of (entry.tags ?? [])) {
        rtn.tags.push(
            cloneInteger(t)
        );
    }

    return rtn;
}

export function makeIssuedByJson(): IssuedByJson {
    return {
        id: null,
        username: "",
        full_name: null
    }
}

export function cloneIssuedByJson(issued_by: IssuedByJson): IssuedByJson {
    return {
        id: cloneInteger(issued_by.id),
        username: cloneString(issued_by.username),
        full_name: optionalCloneString(issued_by.full_name)
    }
}

export function makeCustomFieldJson(): CustomFieldJson {
    return {
        id: null,
        name: "",
        config: makeCustomFieldType(CustomFieldTypeName.Integer),
        comment: null,
        owner: null,
        order: 0,
        issued_by: null
    }
}

export function cloneCustomFieldJson(custom_field: CustomFieldJson): CustomFieldJson {
    return {
        id: cloneInteger(custom_field.id),
        name: cloneString(custom_field.name),
        config: cloneCustomFieldType(custom_field.config),
        comment: optionalCloneString(custom_field.comment),
        owner: cloneInteger(custom_field.owner),
        order: cloneInteger(custom_field.order),
        issued_by: custom_field.issued_by != null ? cloneIssuedByJson(custom_field.issued_by) : null
    }
}

export function makeUserDataJson(): UserDataJson {
    return {
        id: 0,
        username: "",
        level: 20,
        full_name: null,
        email: null,
        email_verified: false
    }
}

export function cloneUserDataJson(user_data: UserDataJson): UserDataJson {
    return {
        id: optionalCloneInteger(user_data.id),
        username: cloneString(user_data.username),
        level: cloneInteger(user_data.level),
        full_name: optionalCloneString(user_data.full_name),
        email: optionalCloneString(user_data.email),
        email_verified: cloneBoolean(user_data.email_verified)
    }
}

export function makeUserAccessInfoJson(): UserAccessInfoJson {
    return {
        id: 0,
        username: "",
        full_name: null,
        ability: null
    }
}

export function cloneUserAccessInfoJson(info: UserAccessInfoJson): UserAccessInfoJson {
    return {
        id: cloneInteger(info.id),
        username: cloneString(info.username),
        full_name: optionalCloneString(info.full_name),
        ability: cloneString(info.ability)
    }
}

export function makeUserInfoJson(): UserInfoJson {
    return {
        id: 0,
        username: "",
        level: 20,
        full_name: null,
        email: null,
        user_access: []
    }
}

export function cloneUserInfoJson(info: UserInfoJson): UserInfoJson {
    let rtn: UserInfoJson = {
        id: cloneInteger(info.id),
        username: cloneString(info.username),
        level: cloneInteger(info.level),
        full_name: optionalCloneString(info.full_name),
        email: optionalCloneString(info.email),
        user_access: []
    };

    for (let item of info.user_access) {
        rtn.user_access.push(cloneUserAccessInfoJson(item));
    }

    return rtn;
}

export function makeTagJson(): TagJson {
    return {
        id: 0,
        title: "",
        color: "#ffffff",
        comment: null,
        owner: 0
    }
}

export function cloneTagJson(tag: TagJson): TagJson {
    return {
        id: cloneInteger(tag.id),
        title: cloneString(tag.title),
        color: cloneString(tag.color),
        comment: optionalCloneString(tag.comment),
        owner: cloneInteger(tag.owner)
    }
}
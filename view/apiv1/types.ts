import { optionalCloneInteger, cloneInteger, optionalCloneString, cloneString, cloneBoolean } from "../util/clone";
import { cloneCustomFieldEntryType, makeCustomFieldEntryType, CustomFieldEntryType } from "./custom_field_entry_types";
import { cloneCustomFieldType, makeCustomFieldType, CustomFieldType, CustomFieldTypeName } from "./custom_field_types";

export interface IssuedByJson {
    id: number
    username: string
    full_name?: string
}

export interface CustomField {
    id: number
    name: string
    config: CustomFieldType
    comment?: string
    owner: number
    order: number
    issued_by?: number
}

export interface CustomFieldEntry {
    field: number
    value: CustomFieldEntryType
    comment?: string
    entry: number
}

export interface TextEntry {
    id: number
    thought: string
    private: boolean
    entry: number
}

export interface EntryMarker {
    id: number
    title: string
    comment?: string
    entry: number
}

export interface Entry {
    id: number
    day: number
    owner: number
}

export interface ComposedEntry {
    entry: Entry
    tags: number[]
    markers: EntryMarker[]
    custom_field_entries: {[id: string]: CustomFieldEntry}
    text_entries: TextEntry[]
}

export interface Tag {
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

export interface PostEntryMarker {
    title: string
    comment?: string
}

export interface PostEntry {
    day: number
}

export interface PostComposedEntry {
    entry: PostEntry
    tags?: number[]
    custom_field_entries?: PostCustomFieldEntry[]
    text_entries?: PostTextEntry[]
    markers?: PostEntryMarker[]
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

export interface PutEntryMarker {
    id?: number
    title: string
    comment?: string
}

export interface PutEntry {
    day: number
}

export interface PutComposedEntry {
    entry: PutEntry
    tags: number[]
    custom_field_entries?: PutCustomFieldEntry[]
    text_entries?: PutTextEntry[]
    markers: PutEntryMarker[]
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

export function makeCustomFieldEntryJson(type: CustomFieldTypeName = CustomFieldTypeName.Integer): CustomFieldEntry {
    return {
        field: 0,
        entry: 0,
        value: makeCustomFieldEntryType(type),
        comment: null
    }
}

export function cloneCustomFieldEntryJson(custom_field_entry: CustomFieldEntry): CustomFieldEntry {
    return {
        field: cloneInteger(custom_field_entry.field),
        entry: cloneInteger(custom_field_entry.entry),
        value: cloneCustomFieldEntryType(custom_field_entry.value),
        comment: optionalCloneString(custom_field_entry.comment)
    };
}

export function makeTextEntry(): TextEntry {
    return {
        id: null,
        thought: "",
        private: false,
        entry: 0
    }
}

export function cloneTextEntry(text_entry: TextEntry): TextEntry {
    return {
        id: optionalCloneInteger(text_entry.id),
        thought: cloneString(text_entry.thought),
        private: cloneBoolean(text_entry.private),
        entry: cloneInteger(text_entry.entry)
    };
}

export function makeEntry(): Entry {
    return {
        id: 0,
        day: 0,
        owner: 0
    }
}

export function cloneEntry(entry: Entry): Entry {
    return {
        id: cloneInteger(entry.id),
        day: cloneInteger(entry.day),
        owner: cloneInteger(entry.owner)
    }
}

export function makeComposedEntry(): ComposedEntry {
    return {
        entry: makeEntry(),
        tags: [],
        markers: [],
        custom_field_entries: {},
        text_entries: []
    }
}

export function cloneComposedEntry(entry: ComposedEntry) {
    let rtn: ComposedEntry = {
        entry: cloneEntry(entry.entry),
        tags: [],
        markers: [],
        custom_field_entries: {},
        text_entries: []
    };

    for (let [k, m] of Object.entries(entry.custom_field_entries)) {
        rtn.custom_field_entries[k] = cloneCustomFieldEntryJson(m);
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

    for (let m of entry.markers) {
        rtn.markers.push(
            cloneEntryMarker(m)
        );
    }

    return rtn;
}

export function makeEntryMarker(): EntryMarker {
    return {
        id: null,
        title: "",
        comment: null,
        entry: 0
    }
}

export function cloneEntryMarker(marker: EntryMarker): EntryMarker {
    return {
        id: optionalCloneInteger(marker.id),
        title: cloneString(marker.title),
        comment: optionalCloneString(marker.comment),
        entry: cloneInteger(marker.entry)
    }
}

export function makeIssuedBy(): IssuedByJson {
    return {
        id: null,
        username: "",
        full_name: null
    }
}

export function cloneIssuedBy(issued_by: IssuedByJson): IssuedByJson {
    return {
        id: cloneInteger(issued_by.id),
        username: cloneString(issued_by.username),
        full_name: optionalCloneString(issued_by.full_name)
    }
}

export function makeCustomField(): CustomField {
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

export function cloneCustomFieldJson(custom_field: CustomField): CustomField {
    return {
        id: cloneInteger(custom_field.id),
        name: cloneString(custom_field.name),
        config: cloneCustomFieldType(custom_field.config),
        comment: optionalCloneString(custom_field.comment),
        owner: cloneInteger(custom_field.owner),
        order: cloneInteger(custom_field.order),
        issued_by: optionalCloneInteger(custom_field.issued_by)
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

export function makeTagJson(): Tag {
    return {
        id: 0,
        title: "",
        color: "#ffffff",
        comment: null,
        owner: 0
    }
}

export function cloneTagJson(tag: Tag): Tag {
    return {
        id: cloneInteger(tag.id),
        title: cloneString(tag.title),
        color: cloneString(tag.color),
        comment: optionalCloneString(tag.comment),
        owner: cloneInteger(tag.owner)
    }
}
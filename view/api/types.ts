import { optionalCloneInteger, cloneInteger, optionalCloneString, cloneString } from "../util/clone";
import { cloneMoodEntryType, makeMoodEntryType, MoodEntryType } from "./mood_entry_types";
import { cloneMoodFieldType, makeMoodFieldType, MoodFieldType, MoodFieldTypeName } from "./mood_field_types";

export interface IssuedByJson {
    id: number
    username: string
    full_name?: string
}

export interface MoodFieldJson {
    id: number
    name: string
    config: MoodFieldType
    comment?: string
    owner: number
    issued_by?: IssuedByJson
}

export interface MoodEntryJson {
    id: number
    field: string
    field_id: number
    value: MoodEntryType
    comment?: string
}

export interface TextEntryJson {
    id: number
    thought: string
}

export interface EntryJson {
    id: number
    created: string
    owner: number
    mood_entries: MoodEntryJson[]
    text_entries: TextEntryJson[]
}

export interface GetEntriesQuery {
    from?: Date | string
    to?: Date | string
}

export interface PostMoodEntry {
    field_id: number
    value: MoodEntryType,
    comment?: string
}

export interface PostTextEntry {
    thought: string
}

export interface PostEntry {
    created?: string
    mood_entries?: PostMoodEntry[]
    text_entries?: PostTextEntry[]
}

export interface PutTextEntry {
    id?: number
    thought: string
}

export interface PutMoodEntry {
    id?: number
    field_id?: number
    value: MoodEntryType
    comment?: string
}

export interface PutEntry {
    created: string,
    mood_entries?: PutMoodEntry[]
    text_entries?: PutTextEntry[]
}

export interface PostMoodField {
    name: string
    config: MoodFieldType
    comment?: string
}

export interface PutMoodField {
    name: string
    config: MoodFieldType
    comment?: string
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
    full_name?: string,
    email?: string
}

export interface PostLogin {
    username: string
    password: string
}

export function makeMoodEntryJson(type: MoodFieldTypeName = MoodFieldTypeName.Integer): MoodEntryJson {
    return {
        id: null,
        field: "",
        field_id: 0,
        value: makeMoodEntryType(type),
        comment: null
    }
}

export function cloneMoodEntryJson(mood_entry: MoodEntryJson): MoodEntryJson {
    return {
        id: optionalCloneInteger(mood_entry.id),
        field: mood_entry.field.slice(0),
        field_id: cloneInteger(mood_entry.field_id),
        value: cloneMoodEntryType(mood_entry.value),
        comment: optionalCloneString(mood_entry.comment)
    };
}

export function makeTextEntry(): TextEntryJson {
    return {
        id: null,
        thought: ""
    }
}

export function cloneTextEntry(text_entry: TextEntryJson): TextEntryJson {
    return {
        id: optionalCloneInteger(text_entry.id),
        thought: cloneString(text_entry.thought)
    };
}

export function makeEntryJson(): EntryJson {
    return {
        id: null,
        created: "",
        owner: 0,
        mood_entries: [],
        text_entries: []
    }
}

export function cloneEntryJson(entry: EntryJson) {
    let rtn: EntryJson = {
        id: optionalCloneInteger(entry.id),
        created: cloneString(entry.created),
        owner: cloneInteger(entry.owner),
        mood_entries: [],
        text_entries: []
    };

    for (let m of (entry.mood_entries ?? [])) {
        rtn.mood_entries.push(
            cloneMoodEntryJson(m)
        );
    }

    for (let t of (entry.text_entries ?? [])) {
        rtn.text_entries.push(
            cloneTextEntry(t)
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

export function makeMoodFieldJson(): MoodFieldJson {
    return {
        id: null,
        name: "",
        config: makeMoodFieldType(MoodFieldTypeName.Integer),
        comment: null,
        owner: null,
        issued_by: null
    }
}

export function cloneMoodFieldJson(mood_field: MoodFieldJson): MoodFieldJson {
    return {
        id: cloneInteger(mood_field.id),
        name: cloneString(mood_field.name),
        config: cloneMoodFieldType(mood_field.config),
        comment: optionalCloneString(mood_field.comment),
        owner: cloneInteger(mood_field.owner),
        issued_by: mood_field.issued_by != null ? cloneIssuedByJson(mood_field.issued_by) : null
    }
}

export function makeUserDataJson(): UserDataJson {
    return {
        id: null,
        username: "",
        full_name: null,
        email: null
    }
}

export function cloneUserDataJson(user_data: UserDataJson): UserDataJson {
    return {
        id: optionalCloneInteger(user_data.id),
        username: cloneString(user_data.username),
        full_name: optionalCloneString(user_data.full_name),
        email: optionalCloneString(user_data.email)
    }
}
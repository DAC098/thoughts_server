import { cloneCustomFieldEntryType, CustomFieldEntryType, makeCustomFieldEntryType } from "./custom_field_entry_types";
import { bool, HashMap, i32, i64, Option, String, Vec } from "../rust_types";
import { cloneCustomFieldType, CustomFieldType, CustomFieldTypeName, makeCustomFieldType } from "./custom_field_types";
import { cloneBoolean, cloneInteger, cloneString, optionalCloneInteger, optionalCloneString } from "../util/clone";

export interface Entry {
    id: i32
    day: i64
    owner: i32
}

export function newEntry(): Entry {
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

export interface EntryMarker {
    id: i32
    title: String
    comment: Option<String>
    entry: i32
}

export function newEntryMarker(): EntryMarker {
    return {
        id: 0,
        title: "",
        comment: null,
        entry: 0
    }
}

export function cloneEntryMarker(entry_marker: EntryMarker): EntryMarker {
    return {
        id: cloneInteger(entry_marker.id),
        title: cloneString(entry_marker.title),
        comment: optionalCloneString(entry_marker.comment),
        entry: cloneInteger(entry_marker.entry)
    }
}

export interface CustomFieldEntry {
    field: i32
    value: CustomFieldEntryType
    comment: Option<String>
    entry: i32
}

export function newCustomFieldEntry(type: CustomFieldTypeName): CustomFieldEntry {
    return {
        field: 0,
        value: makeCustomFieldEntryType(type),
        comment: null,
        entry: 0
    }
}

export function cloneCustomFieldEntry(custom_field_entry: CustomFieldEntry): CustomFieldEntry {
    return {
        field: cloneInteger(custom_field_entry.field),
        value: cloneCustomFieldEntryType(custom_field_entry.value),
        comment: optionalCloneString(custom_field_entry.comment),
        entry: cloneInteger(custom_field_entry.entry)
    }
}

export interface TextEntry {
    id: i32
    thought: String
    private: bool
    entry: i32
}

export function newTextEntry(): TextEntry {
    return {
        id: 0,
        thought: "",
        private: false,
        entry: 0
    }
}

export function cloneTextEntry(text_entry: TextEntry): TextEntry {
    return {
        id: cloneInteger(text_entry.id),
        thought: cloneString(text_entry.thought),
        private: cloneBoolean(text_entry.private),
        entry: cloneInteger(text_entry.entry)
    }
}

export interface AudioEntry {
    id: i32
    private: bool
    entry: i32
}

export function newAudioEntry(): AudioEntry {
    return {
        id: 0,
        private: false,
        entry: 0
    }
}

export function cloneAudioEntry(audio_entry: AudioEntry): AudioEntry {
    return {
        id: cloneInteger(audio_entry.id),
        private: cloneBoolean(audio_entry.private),
        entry: cloneInteger(audio_entry.id)
    }
}

export interface ComposedEntry {
    entry: Entry
    tags: Vec<i32>
    markers: Vec<EntryMarker>
    custom_field_entries: HashMap<i32, CustomFieldEntry>
    text_entries: Vec<TextEntry>
}

export function newComposedEntry(): ComposedEntry {
    return {
        entry: newEntry(),
        tags: [],
        markers: [],
        custom_field_entries: {},
        text_entries: []
    }
}

export function cloneComposedEntry(composed_entry: ComposedEntry): ComposedEntry {
    let rtn =  {
        entry: cloneEntry(composed_entry.entry),
        tags: [],
        markers: [],
        custom_field_entries: {},
        text_entries: []
    };

    for (let tag of composed_entry.tags) {
        rtn.tags.push(cloneInteger(tag));
    }

    for (let marker of composed_entry.markers) {
        rtn.markers.push(cloneEntryMarker(marker));
    }

    for (let field_id in composed_entry.custom_field_entries) {
        rtn.custom_field_entries[field_id] = cloneCustomFieldEntry(composed_entry.custom_field_entries[field_id])
    }

    for (let text_entry of composed_entry.text_entries) {
        rtn.text_entries.push(cloneTextEntry(text_entry));
    }

    return rtn;
}

export interface CustomField {
    id: i32
    name: String
    config: CustomFieldType
    comment: Option<String>
    owner: i32
    order: i32
    issued_by: Option<i32>
}

export function newCustomField(type: CustomFieldTypeName): CustomField {
    return {
        id: 0,
        name: "",
        config: makeCustomFieldType(type),
        comment: null,
        owner: 0,
        order: 0,
        issued_by: null
    }
}

export function cloneCustomField(custom_field: CustomField): CustomField {
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

export interface Tag {
    id: i32
    title: String
    owner: i32
    color: String
    comment: Option<String>
}

export function newTag(): Tag {
    return {
        id: 0,
        title: "",
        owner: 0,
        color: "",
        comment: null
    }
}

export function cloneTag(tag: Tag): Tag {
    return {
        id: cloneInteger(tag.id),
        title: cloneString(tag.title),
        owner: cloneInteger(tag.owner),
        color: cloneString(tag.color),
        comment: optionalCloneString(tag.comment)
    }
}

export enum UserLevel {
    Admin = 1,
    Manager = 10,
    User = 20
}

export interface User {
    id: i32
    username: String
    level: i32
    full_name: Option<String>
    email: Option<String>
    email_verified: bool
}

export interface UserBare {
    id: i32
    username: String
    full_name: Option<String>
}

export function newUser(): User {
    return {
        id: 0,
        username: "",
        level: 20,
        full_name: null,
        email: null,
        email_verified: false
    }
}

export function cloneUser(user: User): User {
    return {
        id: cloneInteger(user.id),
        username: cloneString(user.username),
        level: cloneInteger(user.level),
        full_name: optionalCloneString(user.full_name),
        email: optionalCloneString(user.email),
        email_verified: cloneBoolean(user.email_verified)
    }
}

export function newUserBare(): UserBare {
    return {
        id: 0,
        username: "",
        full_name: null
    }
}

export function cloneUserBare(user: UserBare): UserBare {
    return {
        id: cloneInteger(user.id),
        username: cloneString(user.username),
        full_name: optionalCloneString(user.full_name)
    }
}

export function getUserBareFromUser(user: User): UserBare {
    return {
        id: cloneInteger(user.id),
        username: cloneString(user.username),
        full_name: optionalCloneString(user.full_name)
    }
}

export interface UserAccess {
    owner: i32,
    ability: String,
    allowed_for: i32
}

export function newUserAccess(): UserAccess {
    return {
        owner: 0,
        ability: "",
        allowed_for: 0
    }
}

export function cloneUserAccess(user_access: UserAccess): UserAccess {
    return {
        owner: cloneInteger(user_access.owner),
        ability: cloneString(user_access.ability),
        allowed_for: cloneInteger(user_access.allowed_for)
    }
}

export interface UserData {
    owner: i32,
    prefix: Option<String>,
    suffix: Option<String>,
    first_name: String,
    last_name: String,
    middle_name: Option<String>

    dob: String
}

export function newUserData(): UserData {
    return {
        owner: 0,
        prefix: null,
        suffix: null,
        first_name: "",
        last_name: "",
        middle_name: null,
        dob: ""
    }
}

export function cloneUserData(user_data: UserData): UserData {
    return {
        owner: cloneInteger(user_data.owner),
        prefix: optionalCloneString(user_data.prefix),
        suffix: optionalCloneString(user_data.suffix),
        first_name: cloneString(user_data.first_name),
        last_name: cloneString(user_data.last_name),
        middle_name: optionalCloneString(user_data.middle_name),
        dob: cloneString(user_data.dob)
    }
}

export interface ComposedUser {
    user: User,
    data: UserData
}

export function newComposedUser(): ComposedUser {
    return {
        user: newUser(),
        data: newUserData()
    }
}

export function cloneComposedUser(record: ComposedUser): ComposedUser {
    return {
        user: cloneUser(record.user),
        data: cloneUserData(record.data)
    }
}

export interface ComposedUserAccess {
    user: User,
    access: UserAccess
}

export function newComposedUserAccess(): ComposedUserAccess {
    return {
        user: newUser(),
        access: newUserAccess()
    }
}

export function cloneComposedUserAccess(record: ComposedUserAccess): ComposedUserAccess {
    return {
        user: cloneUser(record.user),
        access: cloneUserAccess(record.access)
    }
}

export interface ComposedFullUser {
    user: User,
    data: UserData,
    access: Vec<ComposedUserAccess>
}

export function newComposedFullUser(): ComposedFullUser {
    return {
        user: newUser(),
        data: newUserData(),
        access: []
    }
}

export function cloneComposedFullUser(record: ComposedFullUser): ComposedFullUser {
    let access = [];

    for (let rec of record.access) {
        access.push(cloneComposedUserAccess(rec));
    }

    return {
        user: cloneUser(record.user),
        data: cloneUserData(record.data),
        access
    }
}

export interface GlobalCustomField {
    id: i32,
    name: String,
    comment: Option<String>,
    config: CustomFieldType
}

export function newGlobalCustomField(type: CustomFieldTypeName): GlobalCustomField {
    return {
        id: 0,
        name: "",
        comment: null,
        config: makeCustomFieldType(type)
    }
}

export function cloneGlobalCustomField(field: GlobalCustomField): GlobalCustomField {
    return {
        id: cloneInteger(field.id),
        name: cloneString(field.name),
        comment: optionalCloneString(field.comment),
        config: cloneCustomFieldType(field.config)
    }
}
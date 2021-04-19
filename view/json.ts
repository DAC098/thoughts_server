import { json } from "./request"

export interface MoodEntryJson {
    id: number
    field: string
    field_id: number
    low: number
    high?: number
    is_range: boolean
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

export interface MoodFieldJson {
    id: number
    name: string
    is_range: boolean
    comment?: string
}

export async function getEntries() {
    let {body} = await json.get<EntryJson[]>("/entries");

    return body.data;
}

export async function getEntry(entry_id: number) {
    let {body} = await json.get<EntryJson>(`/entries/${entry_id}`);

    return body.data;
}

export async function getMoodEntries(entry_id: number) {
    let {body} = await json.get<MoodEntryJson[]>(`/entries/${entry_id}/mood_entries`);

    return body.data;
}

export async function getTextEntries(entry_id: number) {
    let {body} = await json.get<TextEntryJson[]>(`/entries/${entry_id}/text_entries`);

    return body.data;
}

export async function getMoodFields() {
    let {body} = await json.get<MoodFieldJson[]>("/mood_fields");

    return body.data;
}
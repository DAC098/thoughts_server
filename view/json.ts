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
    minimum?: number
    maximum?: number
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

export function makeMoodEntryJson(): MoodEntryJson {
    return {
        id: null,
        field: "",
        field_id: 0,
        low: 0,
        high: null,
        is_range: false,
        comment: null
    }
}

export function cloneMoodEntryJson(mood_entry: MoodEntryJson): MoodEntryJson {
    return {
        id: mood_entry.id != null ? Number(mood_entry.id) : null,
        field: mood_entry.field.slice(0),
        field_id: Number(mood_entry.field_id),
        low: Number(mood_entry.low),
        high: mood_entry.high != null ? Number(mood_entry.high) : null,
        is_range: mood_entry.is_range === true,
        comment: mood_entry.comment?.slice(0) ?? null
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
        id: text_entry.id != null ? Number(text_entry.id) : null,
        thought: text_entry.thought.slice(0)
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
        id: Number(entry.id),
        created: entry.created.slice(0),
        owner: Number(entry.owner),
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

export function makeMoodFieldJson(): MoodFieldJson {
    return {
        id: null,
        name: "",
        minimum: null,
        maximum: null,
        is_range: false,
        comment: null
    }
}

export function cloneMoodFieldJson(mood_field: MoodFieldJson): MoodFieldJson {
    return {
        id: mood_field.id != null ? Number(mood_field.id) : null,
        name: mood_field.name.slice(0),
        minimum: mood_field.minimum != null ? Number(mood_field.minimum) : null,
        maximum: mood_field.maximum != null ? Number(mood_field.maximum) : null,
        is_range: Boolean(mood_field.is_range),
        comment: mood_field.comment != null ? mood_field.comment.slice(0) : null
    }
}

export function compareTextEntryJson(a: TextEntryJson, b: TextEntryJson): boolean {
    return a.thought === b.thought;
}

export function compareMoodEntryJson(a: MoodEntryJson, b: MoodEntryJson): boolean {
    return a.field_id === b.field_id && a.low === b.low && a.high === b.high && a.is_range === b.is_range && a.comment === b.comment;
}

export function compareEntryJson(a: EntryJson, b: EntryJson): boolean {
    if (a.created !== b.created || a.owner !== b.owner) {
        return false;
    }

    if (a.text_entries.length !== b.text_entries.length) {
        return false;
    } else {
        for (let i = 0; i < a.text_entries.length; ++i) {
            if (!compareTextEntryJson(a.text_entries[i], b.text_entries[i])) {
                return false;
            }
        }
    }

    if (a.mood_entries.length !== b.mood_entries.length) {
        return false;
    } else {
        for (let i = 0; i < a.mood_entries.length; ++i) {
            if (!compareMoodEntryJson(a.mood_entries[i], b.mood_entries[i])) {
                return false;
            }
        }
    }

    return true;
}
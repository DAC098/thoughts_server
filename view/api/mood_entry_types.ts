import { unixNow } from "../time";
import { cloneInteger, cloneString } from "../util/clone";
import { MoodFieldTypeName } from "./mood_field_types"

interface MoodEntryTypeBase<T extends string> {
    type: T
}

export interface Integer extends MoodEntryTypeBase<MoodFieldTypeName.Integer> {
    value: number
}

export interface IntegerRange extends MoodEntryTypeBase<MoodFieldTypeName.IntegerRange> {
    low: number
    high: number
}

export interface Float extends MoodEntryTypeBase<MoodFieldTypeName.Float> {
    value: number
}

export interface FloatRange extends MoodEntryTypeBase<MoodFieldTypeName.FloatRange> {
    low: number
    high: number
}

export interface Time extends MoodEntryTypeBase<MoodFieldTypeName.Time> {
    value: string
}

export interface TimeRange extends MoodEntryTypeBase<MoodFieldTypeName.TimeRange> {
    low: string
    high: string
}

export type MoodEntryType = Integer | IntegerRange |
    Float | FloatRange |
    Time | TimeRange;

export function cloneMoodEntryType(entry: MoodEntryType): MoodEntryType {
    switch (entry.type) {
        case MoodFieldTypeName.Integer: {
            return {
                type: MoodFieldTypeName.Integer,
                value: cloneInteger(entry.value)
            }
        }
        case MoodFieldTypeName.Float: {
            return {
                type: MoodFieldTypeName.Float,
                value: cloneInteger(entry.value)
            }
        }
        case MoodFieldTypeName.IntegerRange: {
            return {
                type: MoodFieldTypeName.IntegerRange,
                low: cloneInteger(entry.low),
                high: cloneInteger(entry.high)
            }
        }
        case MoodFieldTypeName.FloatRange: {
            return {
                type: MoodFieldTypeName.FloatRange,
                low: cloneInteger(entry.low),
                high: cloneInteger(entry.high)
            }
        }
        case MoodFieldTypeName.Time: {
            return {
                type: MoodFieldTypeName.Time,
                value: cloneString(entry.value)
            }
        }
        case MoodFieldTypeName.TimeRange: {
            return {
                type: MoodFieldTypeName.TimeRange,
                low: cloneString(entry.low),
                high: cloneString(entry.high)
            }
        }
    }
}

export function makeMoodEntryType(name: MoodFieldTypeName): MoodEntryType {
    switch (name) {
        case MoodFieldTypeName.Integer:
            return {
                type: MoodFieldTypeName.Integer, value: 0
            }
        case MoodFieldTypeName.IntegerRange:
            return {
                type: MoodFieldTypeName.IntegerRange, low: 0, high: 0
            }
        case MoodFieldTypeName.Float: {
            return {
                type: MoodFieldTypeName.Float, value: 0.0
            }
        }
        case MoodFieldTypeName.FloatRange: {
            return {
                type: MoodFieldTypeName.FloatRange, low: 0.0, high: 0.0
            }
        }
        case MoodFieldTypeName.Time: {
            return {
                type: MoodFieldTypeName.Time,
                value: (new Date()).toISOString()
            }
        }
        case MoodFieldTypeName.TimeRange: {
            return {
                type: MoodFieldTypeName.TimeRange, 
                low: (new Date()).toISOString(), 
                high: (new Date()).toISOString()
            }
        }
    }
}
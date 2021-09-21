import { unixNow } from "../util/time";
import { cloneInteger, cloneString } from "../util/clone";
import { CustomFieldTypeName } from "./custom_field_types"

interface CustomFieldEntryTypeBase<T extends string> {
    type: T
}

export interface Integer extends CustomFieldEntryTypeBase<CustomFieldTypeName.Integer> {
    value: number
}

export interface IntegerRange extends CustomFieldEntryTypeBase<CustomFieldTypeName.IntegerRange> {
    low: number
    high: number
}

export interface Float extends CustomFieldEntryTypeBase<CustomFieldTypeName.Float> {
    value: number
}

export interface FloatRange extends CustomFieldEntryTypeBase<CustomFieldTypeName.FloatRange> {
    low: number
    high: number
}

export interface Time extends CustomFieldEntryTypeBase<CustomFieldTypeName.Time> {
    value: string
}

export interface TimeRange extends CustomFieldEntryTypeBase<CustomFieldTypeName.TimeRange> {
    low: string
    high: string
}

export type CustomFieldEntryType = Integer | IntegerRange |
    Float | FloatRange |
    Time | TimeRange;

export function cloneCustomFieldEntryType(entry: CustomFieldEntryType): CustomFieldEntryType {
    switch (entry.type) {
        case CustomFieldTypeName.Integer: {
            return {
                type: CustomFieldTypeName.Integer,
                value: cloneInteger(entry.value)
            }
        }
        case CustomFieldTypeName.Float: {
            return {
                type: CustomFieldTypeName.Float,
                value: cloneInteger(entry.value)
            }
        }
        case CustomFieldTypeName.IntegerRange: {
            return {
                type: CustomFieldTypeName.IntegerRange,
                low: cloneInteger(entry.low),
                high: cloneInteger(entry.high)
            }
        }
        case CustomFieldTypeName.FloatRange: {
            return {
                type: CustomFieldTypeName.FloatRange,
                low: cloneInteger(entry.low),
                high: cloneInteger(entry.high)
            }
        }
        case CustomFieldTypeName.Time: {
            return {
                type: CustomFieldTypeName.Time,
                value: cloneString(entry.value)
            }
        }
        case CustomFieldTypeName.TimeRange: {
            return {
                type: CustomFieldTypeName.TimeRange,
                low: cloneString(entry.low),
                high: cloneString(entry.high)
            }
        }
    }
}

export function makeCustomFieldEntryType(name: CustomFieldTypeName): CustomFieldEntryType {
    switch (name) {
        case CustomFieldTypeName.Integer:
            return {
                type: CustomFieldTypeName.Integer, value: 0
            }
        case CustomFieldTypeName.IntegerRange:
            return {
                type: CustomFieldTypeName.IntegerRange, low: 0, high: 0
            }
        case CustomFieldTypeName.Float: {
            return {
                type: CustomFieldTypeName.Float, value: 0.0
            }
        }
        case CustomFieldTypeName.FloatRange: {
            return {
                type: CustomFieldTypeName.FloatRange, low: 0.0, high: 0.0
            }
        }
        case CustomFieldTypeName.Time: {
            return {
                type: CustomFieldTypeName.Time,
                value: (new Date()).toISOString()
            }
        }
        case CustomFieldTypeName.TimeRange: {
            return {
                type: CustomFieldTypeName.TimeRange, 
                low: (new Date()).toISOString(), 
                high: (new Date()).toISOString()
            }
        }
    }
}
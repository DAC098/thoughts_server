import { cloneBoolean, cloneFloat, cloneInteger, cloneString, optionalCloneInteger } from "../util/clone";

export enum CustomFieldTypeName {
    Integer = "Integer",
    IntegerRange = "IntegerRange",
    Float = "Float",
    FloatRange = "FloatRange",
    Time = "Time",
    TimeRange = "TimeRange"
}

interface CustomFieldTypeBase<T extends string> {
    type: T
}

export interface Integer extends CustomFieldTypeBase<CustomFieldTypeName.Integer> {
    minimum?: number
    maximum?: number
}

export interface IntegerRange extends CustomFieldTypeBase<CustomFieldTypeName.IntegerRange> {
    minimum?: number
    maximum?: number
}

export interface Float extends CustomFieldTypeBase<CustomFieldTypeName.Float> {
    minimum?: number
    maximum?: number
    step: number
    precision: number
}

export interface FloatRange extends CustomFieldTypeBase<CustomFieldTypeName.FloatRange> {
    minimum?: number
    maximum?: number
    step: number
    precision: number
}

export interface Time extends CustomFieldTypeBase<CustomFieldTypeName.Time> {
    as_12hr: boolean
}

export interface TimeRange extends CustomFieldTypeBase<CustomFieldTypeName.TimeRange> {
    show_diff: boolean
    as_12hr: boolean
}

export type CustomFieldType = Integer | IntegerRange |
    Float | FloatRange |
    Time | TimeRange;

export function cloneCustomFieldType(field: CustomFieldType): CustomFieldType {
    switch (field.type) {
        case "Integer":
        case "IntegerRange": {
            return {
                type: <typeof field.type>cloneString(field.type),
                minimum: optionalCloneInteger(field.minimum),
                maximum: optionalCloneInteger(field.maximum),
            }
        }
        case "Float":
        case "FloatRange": {
            return {
                type: <typeof field.type>cloneString(field.type),
                minimum: optionalCloneInteger(field.minimum),
                maximum: optionalCloneInteger(field.maximum),
                step: cloneFloat(field.step),
                precision: cloneInteger(field.precision)
            }
        }
        case "Time": {
            return {
                type: CustomFieldTypeName.Time,
                as_12hr: cloneBoolean(field.as_12hr)
            }
        }
        case "TimeRange": {
            return {
                type: <typeof field.type>cloneString(field.type),
                show_diff: cloneBoolean(field.show_diff),
                as_12hr: cloneBoolean(field.as_12hr)
            }
        }
    }
}

export function makeCustomFieldType(type: CustomFieldTypeName): CustomFieldType {
    switch (type) {
        case "Integer":
            return {type, minimum: null, maximum: null};
        case "IntegerRange":
            return {type, minimum: null, maximum: null};
        case "Float":
            return {type, minimum: null, maximum: null, step: 0.01, precision: 2};
        case "FloatRange":
            return {type, minimum: null, maximum: null, step: 0.01, precision: 2};
        case "Time":
            return {type, as_12hr: false};
        case "TimeRange":
            return {type, show_diff: false, as_12hr: false};
    }
}
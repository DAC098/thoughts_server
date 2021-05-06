import { cloneString, optionalCloneInteger } from "../util/clone";

export enum MoodFieldTypeName {
    Integer = "Integer",
    IntegerRange = "IntegerRange",
    Float = "Float",
    FloatRange = "FloatRange",
    Time = "Time",
    TimeRange = "TimeRange"
}

interface MoodFieldTypeBase<T extends string> {
    type: T
}

export interface Integer extends MoodFieldTypeBase<MoodFieldTypeName.Integer> {
    minimum?: number
    maximum?: number
}

export interface IntegerRange extends MoodFieldTypeBase<MoodFieldTypeName.IntegerRange> {
    minimum?: number
    maximum?: number
}

export interface Float extends MoodFieldTypeBase<MoodFieldTypeName.Float> {
    minimum?: number
    maximum?: number
}

export interface FloatRange extends MoodFieldTypeBase<MoodFieldTypeName.FloatRange> {
    minimum?: number
    maximum?: number
}

export interface Time extends MoodFieldTypeBase<MoodFieldTypeName.Time> {}

export interface TimeRange extends MoodFieldTypeBase<MoodFieldTypeName.TimeRange> {}

export type MoodFieldType = Integer | IntegerRange |
    Float | FloatRange |
    Time | TimeRange;

export function cloneMoodFieldType(field: MoodFieldType): MoodFieldType {
    switch (field.type) {
        case "Integer":
        case "IntegerRange":
        case "Float":
        case "FloatRange": {
            return {
                type: <typeof field.type>cloneString(field.type),
                minimum: optionalCloneInteger(field.minimum),
                maximum: optionalCloneInteger(field.maximum)
            }
        }
        case "Time":
        case "TimeRange": {
            return {
                type: <typeof field.type>cloneString(field.type)
            }
        }
    }
}

export function makeMoodFieldType(type: MoodFieldTypeName): MoodFieldType {
    switch (type) {
        case "Integer":
            return {type, minimum: null, maximum: null};
        case "IntegerRange":
            return {type, minimum: null, maximum: null};
        case "Float":
            return {type, minimum: null, maximum: null};
        case "FloatRange":
            return {type, minimum: null, maximum: null};
        case "Time":
            return {type};
        case "TimeRange":
            return {type};
    }
}
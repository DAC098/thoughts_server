import React from "react"
import { CustomFieldTypeName } from "../../apiv2/custom_field_types"
import { CustomField, ComposedEntry } from "../../apiv2/types"
import FloatGraph from "./Float"
import FloatRangeGraph from "./FloatRange"
import IntegerGraph from "./Integer"
import IntegerRangeGraph from "./IntegerRange"
import TimeGraph from "./Time"
import TimeRangeGraph from "./TimeRange"

interface CustomFieldGraphProps {
    field: CustomField

    entries: ComposedEntry[]

    width: number
    height: number
}

export function CustomFieldGraph({
    field,
    entries,
    width, height
}: CustomFieldGraphProps) {
    switch (field.config.type) {
        case CustomFieldTypeName.Integer:
            return <IntegerGraph width={width} height={height} field={field} entries={entries}/>
        case CustomFieldTypeName.IntegerRange:
            return <IntegerRangeGraph width={width} height={height} field={field} entries={entries}/>
        case CustomFieldTypeName.Float:
            return <FloatGraph width={width} height={height} field={field} entries={entries}/>
        case CustomFieldTypeName.FloatRange:
            return <FloatRangeGraph width={width} height={height} field={field} entries={entries}/>
        case CustomFieldTypeName.Time:
            return <TimeGraph width={width} height={height} field={field} entries={entries}/>
        case CustomFieldTypeName.TimeRange:
            return <TimeRangeGraph width={width} height={height} field={field} entries={entries}/>
    }
}
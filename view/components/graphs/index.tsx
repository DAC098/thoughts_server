import React from "react"
import { CustomFieldTypeName } from "../../api/custom_field_types"
import { CustomFieldJson, EntryJson } from "../../api/types"
import FloatGraph from "./Float"
import FloatRangeGraph from "./FloatRange"
import IntegerGraph from "./Integer"
import IntegerRangeGraph from "./IntegerRange"
import TimeGraph from "./Time"
import TimeRangeGraph from "./TimeRange"

interface CustomFieldGraphProps {
    field: CustomFieldJson

    entries: EntryJson[]

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
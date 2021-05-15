import React from "react"
import { MoodFieldType, MoodFieldTypeName } from "../../api/mood_field_types"
import { FloatRangeEditView } from "./FloatRangeView"
import { FloatEditView } from "./FloatView"
import { IntegerRangeEditView } from "./IntegerRangeView"
import { IntegerEditView } from "./IntegerView"
import { TimeRangeEditView } from "./TimeRangeView"
import { TimeEditView } from "./TimeView"

interface MoodFieldTypeEditViewProps {
    config: MoodFieldType

    onChange?: (config: MoodFieldType) => void
}

export const MoodFieldTypeEditView = ({config, onChange}: MoodFieldTypeEditViewProps) => {
    switch (config.type) {
        case MoodFieldTypeName.Integer:
            return <IntegerEditView config={config} onChange={onChange}/>
        case MoodFieldTypeName.IntegerRange:
            return <IntegerRangeEditView config={config} onChange={onChange}/>
        case MoodFieldTypeName.Float:
            return <FloatEditView config={config} onChange={onChange}/>
        case MoodFieldTypeName.FloatRange:
            return <FloatRangeEditView config={config} onChange={onChange}/>
        case MoodFieldTypeName.Time:
            return <TimeEditView config={config} onChange={onChange}/>
        case MoodFieldTypeName.TimeRange:
            return <TimeRangeEditView config={config} onChange={onChange}/>
    }

    return null;
}
import React from "react"
import { MoodEntryType } from "../../api/mood_entry_types"
import { 
    MoodFieldType, 
    MoodFieldTypeName,
    Integer as IntegerField,
    IntegerRange as IntegerRangeField,
    Float as FloatField,
    FloatRange as FloatRangeField,
    Time as TimeField,
    TimeRange as TimeRangeField
} from "../../api/mood_field_types"
import { FloatRangeEditView, FloatRangeReadView } from "./FloatRangeView"
import { FloatEditView, FloatReadView } from "./FloatView"
import { IntegerRangeEditView, IntegerRangeReadView } from "./IntegerRangeView"
import { IntegerEditView, IntegerReadView } from "./IntegerView"
import { TimeRangeEditView, TimeRangeReadView } from "./TimeRangeView"
import { TimeEditView, TimeReadView } from "./TimeView"

interface MoodEntryTypeEditViewProps {
    value: MoodEntryType
    config?: MoodFieldType

    onChange?: (value: MoodEntryType) => void
}

export const MoodEntryTypeEditView = ({value, config, onChange}: MoodEntryTypeEditViewProps) => {
    switch (value.type) {
        case MoodFieldTypeName.Integer:
            return <IntegerEditView 
                value={value} 
                config={(config as IntegerField)}
                onChange={onChange}
            />
        case MoodFieldTypeName.IntegerRange:
            return <IntegerRangeEditView
                value={value}
                config={(config as IntegerRangeField)}
                onChange={onChange}
            />
        case MoodFieldTypeName.Float:
            return <FloatEditView
                value={value}
                config={(config as FloatField)}
                onChange={onChange}
            />
        case MoodFieldTypeName.FloatRange:
            return <FloatRangeEditView
                value={value}
                config={(config as FloatRangeField)}
                onChange={onChange}
            />
        case MoodFieldTypeName.Time:
            return <TimeEditView
                value={value}
                config={(config as TimeField)}
                onChange={onChange}
            />
        case MoodFieldTypeName.TimeRange:
            return <TimeRangeEditView
                value={value}
                config={(config as TimeRangeField)}
                onChange={onChange}
            />
    }
}

interface MoodEntryTypeReadViewProps {
    value: MoodEntryType
    config?: MoodFieldType
}

export const MoodEntryTypeReadView = ({value, config}: MoodEntryTypeReadViewProps) => {
    switch (value.type) {
        case MoodFieldTypeName.Integer:
            return <IntegerReadView
                value={value} 
                config={(config as IntegerField)}
            />
        case MoodFieldTypeName.IntegerRange:
            return <IntegerRangeReadView
                value={value}
                config={(config as IntegerRangeField)}
            />
        case MoodFieldTypeName.Float:
            return <FloatReadView
                value={value}
                config={(config as FloatField)}
            />
        case MoodFieldTypeName.FloatRange:
            return <FloatRangeReadView
                value={value}
                config={(config as FloatRangeField)}
            />
        case MoodFieldTypeName.Time:
            return <TimeReadView
                value={value}
                config={(config as TimeField)}
            />
        case MoodFieldTypeName.TimeRange:
            return <TimeRangeReadView
                value={value}
                config={(config as TimeRangeField)}
            />
    }
}
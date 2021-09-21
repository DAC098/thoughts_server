import React from "react"
import { CustomFieldEntryType } from "../../apiv2/custom_field_entry_types"
import { 
    CustomFieldType, 
    CustomFieldTypeName,
    Integer as IntegerField,
    IntegerRange as IntegerRangeField,
    Float as FloatField,
    FloatRange as FloatRangeField,
    Time as TimeField,
    TimeRange as TimeRangeField
} from "../../apiv2/custom_field_types"
import { FloatRangeEditView, FloatRangeReadView } from "./FloatRangeView"
import { FloatEditView, FloatReadView } from "./FloatView"
import { IntegerRangeEditView, IntegerRangeReadView } from "./IntegerRangeView"
import { IntegerEditView, IntegerReadView } from "./IntegerView"
import { TimeRangeEditView, TimeRangeReadView } from "./TimeRangeView"
import { TimeEditView, TimeReadView } from "./TimeView"

interface CustomFieldEntryTypeEditViewProps {
    value: CustomFieldEntryType
    config?: CustomFieldType

    onChange?: (value: CustomFieldEntryType) => void
}

export const CustomFieldEntryTypeEditView = ({value, config, onChange}: CustomFieldEntryTypeEditViewProps) => {
    switch (value.type) {
        case CustomFieldTypeName.Integer:
            return <IntegerEditView 
                value={value} 
                config={(config as IntegerField)}
                onChange={onChange}
            />
        case CustomFieldTypeName.IntegerRange:
            return <IntegerRangeEditView
                value={value}
                config={(config as IntegerRangeField)}
                onChange={onChange}
            />
        case CustomFieldTypeName.Float:
            return <FloatEditView
                value={value}
                config={(config as FloatField)}
                onChange={onChange}
            />
        case CustomFieldTypeName.FloatRange:
            return <FloatRangeEditView
                value={value}
                config={(config as FloatRangeField)}
                onChange={onChange}
            />
        case CustomFieldTypeName.Time:
            return <TimeEditView
                value={value}
                config={(config as TimeField)}
                onChange={onChange}
            />
        case CustomFieldTypeName.TimeRange:
            return <TimeRangeEditView
                value={value}
                config={(config as TimeRangeField)}
                onChange={onChange}
            />
    }
}

interface CustomFieldEntryTypeReadViewProps {
    value: CustomFieldEntryType
    config?: CustomFieldType
}

export const CustomFieldEntryTypeReadView = ({value, config}: CustomFieldEntryTypeReadViewProps) => {
    switch (value.type) {
        case CustomFieldTypeName.Integer:
            return <IntegerReadView
                value={value} 
                config={(config as IntegerField)}
            />
        case CustomFieldTypeName.IntegerRange:
            return <IntegerRangeReadView
                value={value}
                config={(config as IntegerRangeField)}
            />
        case CustomFieldTypeName.Float:
            return <FloatReadView
                value={value}
                config={(config as FloatField)}
            />
        case CustomFieldTypeName.FloatRange:
            return <FloatRangeReadView
                value={value}
                config={(config as FloatRangeField)}
            />
        case CustomFieldTypeName.Time:
            return <TimeReadView
                value={value}
                config={(config as TimeField)}
            />
        case CustomFieldTypeName.TimeRange:
            return <TimeRangeReadView
                value={value}
                config={(config as TimeRangeField)}
            />
    }
}
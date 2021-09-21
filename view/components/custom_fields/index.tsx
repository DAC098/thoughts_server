import React from "react"
import { CustomFieldType, CustomFieldTypeName } from "../../apiv2/custom_field_types"
import { FloatRangeEditView } from "./FloatRangeView"
import { FloatEditView } from "./FloatView"
import { IntegerRangeEditView } from "./IntegerRangeView"
import { IntegerEditView } from "./IntegerView"
import { TimeRangeEditView } from "./TimeRangeView"
import { TimeEditView } from "./TimeView"

interface CustomFieldTypeEditViewProps {
    config: CustomFieldType

    onChange?: (config: CustomFieldType) => void
}

export const CustomFieldTypeEditView = ({config, onChange}: CustomFieldTypeEditViewProps) => {
    switch (config.type) {
        case CustomFieldTypeName.Integer:
            return <IntegerEditView config={config} onChange={onChange}/>
        case CustomFieldTypeName.IntegerRange:
            return <IntegerRangeEditView config={config} onChange={onChange}/>
        case CustomFieldTypeName.Float:
            return <FloatEditView config={config} onChange={onChange}/>
        case CustomFieldTypeName.FloatRange:
            return <FloatRangeEditView config={config} onChange={onChange}/>
        case CustomFieldTypeName.Time:
            return <TimeEditView config={config} onChange={onChange}/>
        case CustomFieldTypeName.TimeRange:
            return <TimeRangeEditView config={config} onChange={onChange}/>
    }

    return null;
}
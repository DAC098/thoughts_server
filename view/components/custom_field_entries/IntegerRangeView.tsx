import { SpinButton, Stack, Text } from "@fluentui/react";
import React from "react"
import { IntegerRange } from "../../apiv2/custom_field_entry_types";
import { CustomFieldTypeName, IntegerRange as IntegerRangeField } from "../../apiv2/custom_field_types"

interface DetailsTextProps {
    value: IntegerRange
    config?: IntegerRangeField
}

const DetailsText = ({value, config}: DetailsTextProps) => {
    let detail_text = [
        `type: ${value.type}`
    ];

    if (config != null) {
        if (config.minimum != null) {
            detail_text.push(`minimum: ${config.minimum}`);
        }

        if (config.maximum != null) {
            detail_text.push(`maximum: ${config.maximum}`);
        }
    } else {
        detail_text.push("details unknown")
    }

    return <Text variant="small">{detail_text.join(' | ')}</Text>
}

interface IntegerRangeEditViewProps {
    value: IntegerRange
    config?: IntegerRangeField

    onChange?: (value: IntegerRange) => void
}

export const IntegerRangeEditView = ({value, config = null, onChange}: IntegerRangeEditViewProps) => {

    return <Stack tokens={{childrenGap: 2}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                label="Low"
                value={value.low.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                onChange={(e,v) => {
                    let int = parseInt(v);

                    if (!isNaN(int)) {
                        onChange?.({type: CustomFieldTypeName.IntegerRange, low: int, high: value.high});
                    }
                }}
            />
            <SpinButton
                label="High"
                value={value.high.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                onChange={(e,v) => {
                    let int = parseInt(v);

                    if (!isNaN(int)) {
                        onChange?.({type: CustomFieldTypeName.IntegerRange, low: value.low, high: int});
                    }
                }}
            />
        </Stack>
        <DetailsText value={value} config={config}/>
    </Stack>
}

interface IntegerRangeReadViewProps {
    value: IntegerRange
    config?: IntegerRangeField
}

export const IntegerRangeReadView = ({value, config}: IntegerRangeReadViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Text>{`low: ${value.low} | high: ${value.high}`}</Text>
        <DetailsText value={value} config={config}/>
    </Stack>
}
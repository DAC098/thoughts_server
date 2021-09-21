import { SpinButton, Stack, Text } from "@fluentui/react";
import React from "react"
import { FloatRange } from "../../apiv2/custom_field_entry_types";
import { FloatRange as FloatRangeField, CustomFieldTypeName} from "../../apiv2/custom_field_types"

interface DetailsTextProps {
    value: FloatRange
    config?: FloatRangeField
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

        detail_text.push(`step: ${config.step}`);
        detail_text.push(`precision: ${config.precision}`);
    } else {
        detail_text.push("details unknown");
    }

    return <Text variant="small">{detail_text.join(" | ")}</Text>
}

interface FloatRangeEditViewProps {
    value: FloatRange
    config?: FloatRangeField

    onChange?: (value: FloatRange) => void
}

export const FloatRangeEditView = ({value, config = null, onChange}: FloatRangeEditViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                label="Low"
                value={value.low.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                precision={config.precision}
                step={config.step}
                onChange={(e,v) => {
                    let float = parseFloat(v);

                    if (!isNaN(float) && float <= value.high) {
                        onChange?.({type: CustomFieldTypeName.FloatRange, low: float, high: value.high});
                    }
                }}
            />
            <SpinButton
                label="High"
                value={value.high.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                precision={config.precision}
                step={config.step}
                onChange={(e,v) => {
                    let float = parseFloat(v);

                    if (!isNaN(float) && float >= value.low) {
                        onChange?.({type: CustomFieldTypeName.FloatRange, low: value.low, high: float});
                    }
                }}
            />
        </Stack>
        <DetailsText value={value} config={config}/>
    </Stack>
}

interface FloatRangeReadViewProps {
    value: FloatRange
    config?: FloatRangeField
}

export const FloatRangeReadView = ({value, config}: FloatRangeReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        <Text>{`low: ${value.low} | high: ${value.high}`}</Text>
        <DetailsText value={value} config={config}/>
    </Stack>
}
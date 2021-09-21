import React from "react"
import { SpinButton, Stack, Text } from "@fluentui/react";
import { Float } from "../../apiv2/custom_field_entry_types";
import { Float as FloatField, CustomFieldTypeName} from "../../apiv2/custom_field_types"

interface DetailsTextProps {
    value: Float
    config?: FloatField
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

    return <Text variant="small">{detail_text.join(' | ')}</Text>
}

interface FLoatEditViewProps {
    value: Float
    config?: FloatField

    onChange?: (value: Float) => void
}

export const FloatEditView = ({value, config = null, onChange}: FLoatEditViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                value={value.value.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                step={config.step}
                precision={config.precision}
                onChange={(e,v) => {
                    let float = parseFloat(v);

                    if (!isNaN(float)) {
                        onChange?.({type: CustomFieldTypeName.Float, value: float});
                    }
                }}
            />
        </Stack>
        <DetailsText value={value} config={config}/>
    </Stack>
}

interface FloatReadViewProps {
    value: Float
    config?: FloatField
}

export const FloatReadView = ({value, config}: FloatReadViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Text>{`value: ${value.value}`}</Text>
        <DetailsText value={value} config={config}/>
    </Stack>
}
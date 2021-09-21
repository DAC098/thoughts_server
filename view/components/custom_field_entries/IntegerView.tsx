import { SpinButton, Stack, Text } from "@fluentui/react";
import React from "react"
import { Integer } from "../../apiv2/custom_field_entry_types";
import { Integer as IntegerField, CustomFieldTypeName } from "../../apiv2/custom_field_types"

interface DetailsTextProps {
    value: Integer
    config?: IntegerField
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

interface IntegerEditViewProps {
    value: Integer
    config?: IntegerField

    onChange?: (value: Integer) => void
}

export const IntegerEditView = ({value, config = null, onChange}: IntegerEditViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Stack horizontal tokens={{childrenGap: 8}}>
            <SpinButton
                label="Value"
                value={value.value.toString()}
                min={config != null ? config.minimum : null}
                max={config != null ? config.maximum : null}
                onChange={(e, v) => {
                    let int = parseInt(v);

                    if (!isNaN(int)) {
                        onChange?.({type: CustomFieldTypeName.Integer, value: int})
                    }
                }}
            />
        </Stack>
        <DetailsText value={value} config={config}/>
    </Stack>
}

interface IntegerReadViewProps {
    value: Integer
    config?: IntegerField
}

export const IntegerReadView = ({value, config}: IntegerReadViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Text>{`value: ${value.value}`}</Text>
        <DetailsText value={value} config={config}/>
    </Stack>;
}
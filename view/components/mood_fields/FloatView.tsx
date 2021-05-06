import { Label, SpinButton, Stack, Toggle } from "@fluentui/react";
import React from "react"
import { Float, MoodFieldTypeName } from "../../api/mood_field_types";

interface FloatEditViewProps {
    config: Float

    onChange?: (config: Float) => void
}

export const FloatEditView = ({config, onChange}: FloatEditViewProps) => {
    return <Stack horizontal tokens={{childrenGap: 8}}>
        <Stack tokens={{childrenGap: 8}}>
            <Label>Minimum</Label>
            <Stack horizontal verticalAlign="center" tokens={{childrenGap: 8}}>
                <Toggle checked={config.minimum != null} onChange={(e,c) => {
                    onChange?.({type: MoodFieldTypeName.Float, minimum: c ? 0 : null, maximum: config.maximum})
                }}/>
                <SpinButton
                    disabled={config.minimum == null}
                    value={config.minimum?.toString() ?? "0"}
                    onChange={(e,v) => {
                        let float = parseFloat(v);

                        if (!isNaN(float) && (config.maximum != null ? float < config.maximum : true)) {
                            onChange?.({type: MoodFieldTypeName.Float, minimum: float, maximum: config.maximum});
                        }
                    }}
                />
            </Stack>
        </Stack>
        <Stack tokens={{childrenGap: 8}}>
            <Label>Maximum</Label>
            <Stack horizontal verticalAlign="center" tokens={{childrenGap: 8}}>
                <Toggle checked={config.maximum != null} onChange={(e,c) => {
                    onChange?.({type: MoodFieldTypeName.Float, minimum: config.minimum, maximum: c ? 0 : null});
                }}/>
                <SpinButton
                    disabled={config.maximum == null}
                    value={config.maximum?.toString() ?? "0"}
                    onChange={(e,v) => {
                        let float = parseFloat(v);

                        if (!isNaN(float) && (config.minimum != null ? float > config.minimum : true)) {
                            onChange?.({type: MoodFieldTypeName.Float, minimum: config.minimum, maximum: float});
                        }
                    }}
                />
            </Stack>
        </Stack>
    </Stack>
}
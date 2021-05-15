import React from "react"
import { Stack, Toggle } from "@fluentui/react";
import { MoodFieldTypeName, Time } from "../../api/mood_field_types";

interface TimeEditViewProps {
    config: Time,
    
    onChange?: (config: Time) => void
}

export const TimeEditView = ({config, onChange}: TimeEditViewProps) => {
    return <Stack horizontal tokens={{childrenGap: 8}}>
        <Toggle label="As 12hr clock" onText="On" offText="Off" checked={config.as_12hr} onChange={(e,c) => 
            onChange?.({type: MoodFieldTypeName.Time, as_12hr: c})
        }/>
    </Stack>
}

interface TimeReadViewProps {

}

export const TimeReadView = ({}: TimeReadViewProps) => {
    return <div>TimeRead</div>
}
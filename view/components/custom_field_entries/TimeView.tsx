import { DatePicker, DefaultButton, Label, Stack, Text, TooltipHost } from "@fluentui/react";
import React from "react"
import { Time } from "../../apiv2/custom_field_entry_types";
import { CustomFieldTypeName, Time as TimeField } from "../../apiv2/custom_field_types";
import { displayDate } from "../../util/time";
import TimeInput from "../TimeInput";

interface DetailsTextProps {
    value: Time
    config?: TimeField
}

const DetailsText = ({value, config}: DetailsTextProps) => {
    let detail_text = [
        `type: ${value.type}`
    ];

    if (config != null) {
    } else {
        detail_text.push("details unknown");
    }

    return <Text variant="small">{detail_text.join(" | ")}</Text>
}

interface TimeEditViewProps {
    value: Time
    config?: TimeField

    onChange?: (value: Time) => void
}

export const TimeEditView = ({value, config = null, onChange}: TimeEditViewProps) => {
    let date = new Date(value.value);

    return <Stack tokens={{childrenGap: 2}}>
        <Stack tokens={{childrenGap: 2}}>
            <Label>Value</Label>
            <Stack horizontal tokens={{childrenGap: 8}}>
                <DatePicker
                    value={date}
                    onSelectDate={d => {
                        let copy = new Date(date.getTime());
                        copy.setFullYear(d.getFullYear());
                        copy.setMonth(d.getMonth());
                        copy.setDate(d.getDate());
                        onChange?.({type: CustomFieldTypeName.Time, value: copy.toISOString()});
                    }}
                />
                <TimeInput value={date} onChange={d => {
                    onChange?.({type: CustomFieldTypeName.Time, value: d.toISOString()});
                }}/>
                <TooltipHost content="sets the time to this current moment">
                    <DefaultButton text="Now" onClick={() => onChange?.({type: CustomFieldTypeName.Time, value: (new Date()).toISOString()})}/>
                </TooltipHost>
            </Stack>
        </Stack>
        <DetailsText value={value} config={config}/>
    </Stack>
}

interface TimeReadViewProps {
    value: Time
    config?: TimeField
}

export const TimeReadView = ({value, config}: TimeReadViewProps) => {
    return <Stack tokens={{childrenGap: 2}}>
        <Text>{`value: ${displayDate(new Date(value.value))}`}</Text>
        <DetailsText value={value} config={config}/>
    </Stack>
}
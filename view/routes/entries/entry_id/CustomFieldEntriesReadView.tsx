import { Stack, Separator, Label, Text } from "@fluentui/react"
import React from "react"
import { CustomFieldEntry, CustomField } from "../../../apiv2/types"
import { CustomFieldEntryTypeReadView } from "../../../components/custom_field_entries"

interface CustomFieldEntriesReadViewProps {
    custom_field_entries: {[id: string]: CustomFieldEntry}
    custom_fields: CustomField[]
}

const CustomFieldEntriesReadView = ({custom_fields, custom_field_entries}: CustomFieldEntriesReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {custom_fields.filter(field => field.id in custom_field_entries).map((field) => 
            <Stack key={field.id} tokens={{childrenGap: 8}}>
                <Stack tokens={{childrenGap: 8}}>
                    <Separator alignContent="start">
                        <Label title={field.comment}>{field.name}</Label>
                    </Separator>
                    <CustomFieldEntryTypeReadView 
                        value={custom_field_entries[field.id].value}
                        config={field.config}
                    />
                </Stack>
                <Text>{custom_field_entries[field.id].comment}</Text>
            </Stack>
        )}
    </Stack>
}

export default CustomFieldEntriesReadView;
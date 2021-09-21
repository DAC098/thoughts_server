import { Stack, Separator, Label, IconButton, TextField } from "@fluentui/react";
import React, { useContext } from "react";
import { CustomFieldEntryType } from "../../../apiv2/custom_field_entry_types";
import { CustomField, CustomFieldEntry } from "../../../apiv2/types";
import { CustomFieldEntryTypeEditView } from "../../../components/custom_field_entries";
import { EntryIdViewContext, entry_id_view_actions } from "./reducer";

interface CustomFieldEntryInputProps {
    field: CustomField
    entry: CustomFieldEntry

    onDelete?: () => void
    onChange?: (entry: {comment: string, value: CustomFieldEntryType}) => void
}

const CustomFieldEntryInputs = ({
    field,
    entry,
    onDelete,
    onChange
}: CustomFieldEntryInputProps) => {
    let similar_types = entry.value.type === field.config.type;

    return <Stack tokens={{childrenGap: 8}}>
        <Separator alignContent="start">
            <Stack horizontal tokens={{childrenGap: 8}}>
                <Label title={field.comment}>{field.name}</Label>
                <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                    onDelete?.();
                }}/>
            </Stack>
        </Separator>
        <CustomFieldEntryTypeEditView value={entry.value} config={similar_types ? field.config : null} onChange={value => {
            onChange?.({comment: entry.comment, value});
        }}/>
        <TextField type="text" placeholder="comment" value={entry.comment ?? ""} onChange={(e,v) => {
            onChange?.({comment: v, value: entry.value});
        }}/>
    </Stack>
}

interface CustomFieldEntriesEditViewProps {
    custom_fields: CustomField[]
    custom_field_entries: {[id: string]: CustomFieldEntry}
}

const CustomFieldEntriesEditView = ({custom_fields, custom_field_entries}: CustomFieldEntriesEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {custom_fields.filter(field => field.id in custom_field_entries).map((field, index) =>
            <CustomFieldEntryInputs
                key={field.id}
                field={field}
                entry={custom_field_entries[field.id]}
        
                onDelete={() => dispatch(entry_id_view_actions.delete_custom_field_entry(field.id.toString()))}
                onChange={(value) => dispatch(entry_id_view_actions.update_custom_field_entry({index: field.id, ...value}))}
            />
        )}
    </Stack>
}

export default CustomFieldEntriesEditView;
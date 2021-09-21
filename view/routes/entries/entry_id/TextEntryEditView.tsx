import { Stack, TextField, Toggle, IconButton } from "@fluentui/react";
import React, { useContext } from "react";
import { TextEntryUI, EntryIdViewContext, entry_id_view_actions } from "./reducer";

interface TextEntryEditViewProps {
    text_entries: TextEntryUI[]
}

const TextEntryEditView = ({text_entries}: TextEntryEditViewProps) => {
    let dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v, index) => {
            return <Stack key={v.key ?? v.id} tokens={{childrenGap: 8}}>
                <TextField multiline autoAdjustHeight value={v.thought} onChange={(e, thought) => {
                    dispatch(entry_id_view_actions.update_text_entry({index, thought, private: v.private}));
                }}/>
                <Stack horizontal tokens={{childrenGap: 8}}>
                    <Toggle label="Private" inlineLabel onText="Yes" offText="No" checked={v.private} onChange={(e,checked) => {
                        dispatch(entry_id_view_actions.update_text_entry({index, thought: v.thought, private: checked}))
                    }}/>
                    <IconButton iconProps={{iconName: "Delete"}} onClick={() => {
                        dispatch(entry_id_view_actions.delete_text_entry(index));
                    }}/>
                </Stack>
            </Stack>
        })}
    </Stack>
}

export default TextEntryEditView;
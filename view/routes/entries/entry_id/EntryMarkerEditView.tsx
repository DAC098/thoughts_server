import { Stack, Separator, Label, TextField, IconButton } from "@fluentui/react";
import React, { useContext } from "react";
import { EntryMarkerUI, EntryIdViewContext, entry_id_view_actions } from "./reducer";

interface EntryMarkerEditViewProps {
    markers: EntryMarkerUI[]
}

const EntryMarkerEditView = ({markers}: EntryMarkerEditViewProps) => {
    const dispatch = useContext(EntryIdViewContext);

    return <Stack tokens={{childrenGap: 8}}>
        {markers.length > 0 ?
            <Separator alignContent="start">
                <Label>Markers</Label>
            </Separator>
            :
            null
        }
        {markers.map((marker, index) =>
            <Stack key={marker.id ?? marker.key} horizontal tokens={{childrenGap: 8}} verticalAlign="end">
                <TextField
                    label="Title"
                    type="text"
                    value={marker.title}
                    onChange={(e, v) => 
                        dispatch(entry_id_view_actions.update_entry_marker({index, title: v, comment: marker.comment}))
                    }
                />
                <TextField
                    label="Comment"
                    type="text"
                    value={marker.comment ?? ""}
                    styles={{root: {flex: 1}}}
                    onChange={(e, v) =>
                        dispatch(entry_id_view_actions.update_entry_marker({index, title: marker.title, comment: v}))
                    }
                />
                <IconButton iconProps={{iconName: "Delete"}} onClick={() =>
                    dispatch(entry_id_view_actions.delete_entry_marker(index))
                }/>
            </Stack>
        )}
    </Stack>
}

export default EntryMarkerEditView;
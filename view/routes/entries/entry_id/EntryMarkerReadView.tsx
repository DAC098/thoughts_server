import { Stack, Separator, Label, Text } from "@fluentui/react"
import React from "react"
import { EntryMarkerUI } from "./reducer"

interface EntryMarkerReadViewProps {
    markers: EntryMarkerUI[]
}

const EntryMarkerReadView = ({markers}: EntryMarkerReadViewProps) => {
    console.log(markers);
    
    return <Stack tokens={{childrenGap: 8}}>
        {markers.length > 0 ?
            <Separator alignContent="start">
                <Label>Markers</Label>
            </Separator>
            :
            null
        }
        {markers.map((marker) =>
            <Stack key={marker.id ?? marker.key} horizontal tokens={{childrenGap: 8}} verticalAlign="end">
                <Text>{marker.title}</Text>
                <Text variant="small">{marker.comment?.length ? marker.comment : ""}</Text>
            </Stack>
        )}
    </Stack>
}

export default EntryMarkerReadView;
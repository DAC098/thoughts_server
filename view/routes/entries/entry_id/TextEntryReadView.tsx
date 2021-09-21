import { Stack, Text } from "@fluentui/react";
import React from "react";
import { TextEntry } from "../../../apiv2/types";

interface TextEntryReadViewProps {
    text_entries: TextEntry[]
}

const TextEntryReadView = ({text_entries}: TextEntryReadViewProps) => {
    return <Stack tokens={{childrenGap: 8}}>
        {text_entries.map((v) => {
            let line_splits = v.thought.split(/\n/);
            let total = line_splits.length;

            return <div key={v.id}>
                {line_splits.map((t, i) =>
                    <div key={i} style={{paddingBottom: total - 1 !== i ? 4 : 0}}>
                        <Text>{t}</Text>
                    </div>
                )}
            </div>
        })}
    </Stack>
}

export default TextEntryReadView;
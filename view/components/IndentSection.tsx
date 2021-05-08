import { ISeparatorProps, IStackProps, Separator, Stack } from "@fluentui/react";
import React from "react";

interface IndentSectionProps {
    content: React.ReactNode
    indent?: number

    separator?: ISeparatorProps
    stack?: IStackProps

    children?: React.ReactNode
}

const IndentSection = ({
    content,
    indent = 12,
    separator = {"alignContent": "start"}, 
    stack = {},
    children
}: IndentSectionProps) => {
    let tokens = {
        childrenGap: 8,
        ...(stack.tokens ?? {}),
        padding: `0 0 0 ${indent}px`
    };
    return <>
        <Separator {...separator} children={content}/>
        <Stack {...stack} tokens={tokens} children={children}/>
    </>
}

export default IndentSection;
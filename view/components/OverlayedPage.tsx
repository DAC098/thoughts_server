import { IScrollablePaneProps, IStackProps, ScrollablePane, Stack } from "@fluentui/react"
import React, { ReactNode } from "react"

interface OverlayedPageProps {
    width?: number | string
    height?: number | string

    overlayColor?: string
    backgroundColor?: string

    rootStack?: IStackProps
    innerStack?: IStackProps
    scrollablePane?: IScrollablePaneProps

    children?: ReactNode
}

const OverlayedPage = ({
    width = 600, height = "100%",
    overlayColor = "rgba(0,0,0,0.5)", backgroundColor = "white",
    rootStack = {}, innerStack = {},
    scrollablePane = {},
    children
}: OverlayedPageProps) => {
    return <Stack
        {...rootStack}
        horizontal={rootStack?.horizontal ?? true}
        verticalAlign={rootStack?.verticalAlign ?? "center"}
        horizontalAlign={rootStack?.horizontalAlign ?? "center"}
        styles={{root: {
            width: "100%", height: "100%",
            backgroundColor: overlayColor,
            position: "absolute",
            top: 0,
            zIndex: 1
        }}}
    >
        <Stack
            {...innerStack}
            styles={{root: {
                width, height,
                backgroundColor,
                position: "relative"
            }}}
            tokens={{maxWidth: "100%"}}
        >
            <ScrollablePane {...scrollablePane}>
                {children}
            </ScrollablePane>
        </Stack>
    </Stack>
}

export default OverlayedPage;
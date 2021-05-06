import { Persona, Stack } from "@fluentui/react"
import React from "react"

const UserId = () => {
    return <Stack
        horizontal
        verticalAlign="center"
        horizontalAlign="center"
        style={{
            width: "100%", height: "100%",
            backgroundColor: "rgba(0,0,0,0.5)",
            position: "absolute",
            top: 0,
            zIndex: 1
        }}
    >
        <Stack
            style={{
                position: "relative",
                width: 450, height: "100%",
                backgroundColor: "white"
            }}
        >
            <Persona
                text=""
                secondaryText={""}
            />
        </Stack>
    </Stack>
}

export default UserId;
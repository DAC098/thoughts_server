import React from "react"
import { getBrightness, min_brightness } from "../../util/colors"

interface TagTokenProps {
    color: string
    title: string

    display?: string
    padding?: number | string

    fontSize?: number | string
    lineHeight?: number | string

    lightColor?: string
    darkColor?: string
}

const TagToken = ({
    color, title,
    display = "inline-block",
    padding = "0 8px",
    fontSize = 14, lineHeight = 26,
    lightColor = "white", darkColor = "black"
}: TagTokenProps) => <div style={{
    padding, display,
    fontSize,
    lineHeight: typeof lineHeight === "number" ? lineHeight.toString() + "px" : lineHeight,
    backgroundColor: color,
    color: getBrightness(color) < min_brightness ? lightColor : darkColor,
}} children={title}/>

export default TagToken;
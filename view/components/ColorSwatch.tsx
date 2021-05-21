import React from "react"
import { getBrightness, min_brightness } from "../util/colors"

interface ColorSwatchProps {
    color: string
    size?: number
    display?: string
    borderWidth?: number

    lightColor?: string
    darkColor?: string
}

const ColorSwatch = ({
    color,
    size = 20,
    display = "block",
    borderWidth = 1,
    lightColor = "white", darkColor = "black"
}: ColorSwatchProps) => <div style={{
    width: size, height: size,
    borderRadius: size / 2,
    backgroundColor: color,
    border: `${borderWidth}px solid ${getBrightness(color) < min_brightness ? lightColor : darkColor}`,
    display: display,
    boxSizing: "border-box"
}}/>

export default ColorSwatch;
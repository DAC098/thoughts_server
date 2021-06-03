import React from "react"
import { LinePath } from "@visx/shape"
import { CurveFactory, CurveFactoryLineOnly } from "d3-shape"

interface LinePathProps<T> {
    data: T[]

    xGetter: (d: T) => number
    yGetter: (d: T) => number

    curve?: CurveFactory | CurveFactoryLineOnly

    marker?: string
    markerStart?: string
    markerEnd?: string
}

export function DashedLinePath<T = any>({
    data,
    xGetter, yGetter,
    curve,
    marker,
    markerStart, markerEnd
}: LinePathProps<T>) {
    return <LinePath
        data={data}
        curve={curve}
        x={xGetter}
        y={yGetter}
        stroke="#222"
        strokeWidth={1.5}
        strokeOpacity={0.8}
        strokeDasharray="1,3"
        markerMid={marker}
        markerStart={markerStart ?? marker}
        markerEnd={markerEnd ?? marker}
    />
}

export function SolidLinePath<T = any>({
    data,
    xGetter, yGetter,
    curve,
    marker,
    markerStart, markerEnd
}: LinePathProps<T>) {
    return <LinePath
        data={data}
        curve={curve}
        x={xGetter}
        y={yGetter}
        stroke="#222"
        strokeWidth={1.5}
        markerMid={marker}
        markerStart={markerStart ?? marker}
        markerEnd={markerEnd ?? marker}
    />
}
import React, { Fragment } from "react"
import { Group } from '@visx/group'
import { AxisLeft, AxisBottom } from '@visx/axis'
import { GridRows, GridColumns } from '@visx/grid'
import { scaleTime, scaleLinear } from '@visx/scale'
import * as CurveType from "@visx/curve"
import { Integer } from "../../apiv2/custom_field_entry_types";
import { CustomField, ComposedEntry } from "../../apiv2/types";
import { CircleMarker, TransCircleMarker } from "./markers"
import { DashedLinePath, SolidLinePath } from "./line_paths"
import { defaultGetX } from "./getters"
import { entryIterator } from "./util"

export const background = '#f3f3f3';

const getY = (entry: ComposedEntry, field_id: string) => {
    return (entry.custom_field_entries[field_id].value as Integer).value;
}

const defaultMargin = { top: 40, right: 30, bottom: 50, left: 40 };

interface IntegerGraphProps {
    width: number
    height: number
    margin?: { top: number; right: number; bottom: number; left: number }

    field: CustomField

    entries?: ComposedEntry[]
}

export default function IntegerGraph({
    width, height,
    margin = defaultMargin,
    field,
    entries = []
}: IntegerGraphProps) {
    if (width < 10) return null;

    let field_id = field.id.toString();

    const {
        min_x, min_y,
        max_x, max_y,
        data_groups
    } = entryIterator<Integer>(entries, field, (rtn, entry, field, value) => {
        if (rtn.min_y > value.value) {
            rtn.min_y = value.value;
        }

        if (rtn.max_y < value.value) {
            rtn.max_y = value.value;
        }
    })

    const y_axis_scale = scaleLinear<number>({
        domain:[min_y, max_y]
    });
    const x_axis_scale = scaleTime<number>({
        domain: [min_x, max_x]
    });

    // bounds
    const xMax = width - margin.left - margin.right;
    const yMax = height - margin.top - margin.bottom;

    y_axis_scale.range([yMax, 0]);
    x_axis_scale.range([0, xMax]);

    return (
    <svg width={width} height={height}>
        <CircleMarker/>
        <TransCircleMarker/>
        <rect x={0} y={0} width={width} height={height} fill={background} rx={14}/>
        <Group left={margin.left} top={margin.top}>
            <GridRows scale={y_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
            <GridColumns scale={x_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
            <line x1={xMax} x2={xMax} y1={0} y2={yMax} stroke="#e0e0e0"/>
            <AxisBottom top={yMax} scale={x_axis_scale} numTicks={width > 520 ? 10 : 5}/>
            <AxisLeft scale={y_axis_scale} tickFormat={(value) => value.toString()}/>
            {data_groups.map(set => {
                return <Fragment key={Math.random()}>
                    <DashedLinePath
                        data={set}
                        xGetter={d => x_axis_scale(defaultGetX(d))}
                        yGetter={d => y_axis_scale(getY(d, field_id))}
                        marker={TransCircleMarker.url}
                    />
                    <SolidLinePath
                        data={set}
                        curve={CurveType.curveBasis}
                        xGetter={d => x_axis_scale(defaultGetX(d))}
                        yGetter={d => y_axis_scale(getY(d, field_id))}
                        marker={CircleMarker.url}
                    />
                </Fragment>
            })}
        </Group>
    </svg>
    )
}
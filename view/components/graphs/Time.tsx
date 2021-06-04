import React, { Fragment } from "react"
import { Group } from '@visx/group'
import { curveBasis } from '@visx/curve'
import { LinePath } from '@visx/shape'
import { AxisLeft, AxisBottom } from '@visx/axis'
import { GridRows, GridColumns } from '@visx/grid'
import { scaleTime, scaleLinear } from '@visx/scale'
import { Time } from "../../api/custom_field_entry_types";
import { CustomFieldJson, EntryJson } from "../../api/types";
import { SolidLinePath } from "./line_paths"
import { CircleMarker } from "./markers"

export const background = '#f3f3f3';

const getY = (entry: EntryJson, field_id: string) => {
    return new Date((entry.custom_field_entries[field_id].value as Time).value).getTime();
}

const getX = (entry: EntryJson) => {
    return new Date(entry.created).getTime();
}

const defaultMargin = { top: 40, right: 30, bottom: 50, left: 40 };

interface TimeGraphProps {
    width: number
    height: number
    margin?: { top: number; right: number; bottom: number; left: number }

    field: CustomFieldJson

    entries?: EntryJson[]
}

export default function TimeGraph({
    width, height,
    margin = defaultMargin,
    field,
    entries = []
}: TimeGraphProps) {
    if (width < 10) return null;

    let min_y_domain = Infinity;
    let max_y_domain = -Infinity;
    let min_x_domain = Infinity;
    let max_x_domain = -Infinity;
    let field_id = field.id.toString();
    let data_groups: EntryJson[][] = [];
    let field_entries: EntryJson[] = [];

    for (let entry of entries) {
        let date = new Date(entry.created).getTime();

        if (min_x_domain > date) {
            min_x_domain = date;
        }

        if (max_x_domain < date) {
            max_x_domain = date;
        }
        
        if (field_id in entry.custom_field_entries) {
            let value = new Date((entry.custom_field_entries[field_id].value as Time).value).getTime();

            if (min_y_domain > value) {
                min_y_domain = value;
            }

            if (max_y_domain < value) {
                max_y_domain = value;
            }

            field_entries.push(entry);
        } else {
            if (field_entries.length > 1) {
                data_groups.push(field_entries.slice());
                field_entries = [];
            }
        }
    }

    if (field_entries.length > 1) {
        data_groups.push(field_entries);
        field_entries = [];
    }

    const y_axis_scale = scaleTime<number>({
        domain:[min_y_domain, max_y_domain]
    });
    const x_axis_scale = scaleTime<number>({
        domain: [min_x_domain, max_x_domain]
    });

    // bounds
    const xMax = width - margin.left - margin.right;
    const yMax = height - margin.top - margin.bottom;

    y_axis_scale.range([yMax, 0]);
    x_axis_scale.range([0, xMax]);

    return (
    <svg width={width} height={height}>
        <CircleMarker/>
        <rect x={0} y={0} width={width} height={height} fill={background} rx={14}/>
        <Group left={margin.left} top={margin.top}>
            <GridRows scale={y_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
            <GridColumns scale={x_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
            <line x1={xMax} x2={xMax} y1={0} y2={yMax} stroke="#e0e0e0"/>
            <AxisBottom top={yMax} scale={x_axis_scale} numTicks={width > 520 ? 10 : 5}/>
            <AxisLeft scale={y_axis_scale}/>
            {data_groups.map(set => {
                return <Fragment key={Math.random()}>
                    <SolidLinePath
                        data={set}
                        xGetter={d => x_axis_scale(getX(d))}
                        yGetter={d => y_axis_scale(getY(d, field_id))}
                        marker={CircleMarker.url}
                    />
                </Fragment>
            })}
        </Group>
    </svg>
    )
}
import React, { Fragment } from "react"
import { Group } from '@visx/group'
import { AxisLeft, AxisBottom } from '@visx/axis'
import { GridRows, GridColumns } from '@visx/grid'
import { scaleTime, scaleLinear } from '@visx/scale'
import { Float } from "../../api/custom_field_entry_types";
import { CustomFieldJson, EntryJson } from "../../api/types";
import { CircleMarker, TransCircleMarker } from "./markers"
import { SolidLinePath } from "./line_paths"
import { defaultGetX } from "./getters"
import { getDateZeroHMSM } from "../../util/time"

export const background = '#f3f3f3';

const getY = (entry: EntryJson, field_id: string) => {
    return (entry.custom_field_entries[field_id].value as Float).value;
}

const defaultMargin = { top: 40, right: 30, bottom: 50, left: 40 };

interface IntegerGraphProps {
    width: number
    height: number
    margin?: { top: number; right: number; bottom: number; left: number }

    field: CustomFieldJson

    entries?: EntryJson[]
}

export default function FloatGraph({
    width, height,
    margin = defaultMargin,
    field,
    entries = []
}: IntegerGraphProps) {
    if (width < 10) return null;

    let min_y_domain = Infinity;
    let max_y_domain = -Infinity;
    let min_x_domain = Infinity;
    let max_x_domain = -Infinity;
    let field_id = field.id.toString();
    let data_groups: EntryJson[][] = [];
    let field_entries: EntryJson[] = [];

    for (let entry of entries) {
        let date = getDateZeroHMSM(entry.created).getTime();

        if (min_x_domain > date) {
            min_x_domain = date;
        }

        if (max_x_domain < date) {
            max_x_domain = date;
        }
        
        if (field_id in entry.custom_field_entries) {
            let value = entry.custom_field_entries[field_id].value as Float;

            if (min_y_domain > value.value) {
                min_y_domain = value.value;
            }

            if (max_y_domain < value.value) {
                max_y_domain = value.value;
            }

            field_entries.push(entry);
        } else {
            if (field_entries.length > 0) {
                data_groups.push(field_entries.slice());
                field_entries = [];
            }
        }
    }

    if (field_entries.length > 0) {
        data_groups.push(field_entries);
        field_entries = [];
    }

    const y_axis_scale = scaleLinear<number>({
        domain:[min_y_domain, max_y_domain],
        nice: true
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
        <TransCircleMarker/>
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
                        xGetter={d => x_axis_scale(defaultGetX(d))}
                        yGetter={d => y_axis_scale(getY(d, field_id))}
                        marker={TransCircleMarker.url}
                    />
                </Fragment>
            })}
        </Group>
    </svg>
    )
}
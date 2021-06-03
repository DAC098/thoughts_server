import React, { Fragment } from 'react'
import { Group } from '@visx/group'
import { Threshold } from '@visx/threshold'
import { scaleTime, scaleLinear } from '@visx/scale'
import { AxisLeft, AxisBottom } from '@visx/axis'
import { GridRows, GridColumns } from '@visx/grid'
import { FloatRange } from "../../api/custom_field_entry_types"
import { CustomFieldJson, EntryJson } from '../../api/types'
import { DashedLinePath, SolidLinePath } from './line_paths'
import { CircleMarker } from './markers'

export const background = '#f3f3f3';

const getY0 = (entry: EntryJson, field_id: string) => {
    return (entry.custom_field_entries[field_id].value as FloatRange).low;
}

const getY1 = (entry: EntryJson, field_id: string) => {
    return (entry.custom_field_entries[field_id].value as FloatRange).high;
}

const getX = (entry: EntryJson) => {
    return new Date(entry.created).getTime();
}

const defaultMargin = { top: 40, right: 30, bottom: 50, left: 40 };

export type FloatRangeGraphProps = {
    width: number
    height: number
    margin?: { top: number; right: number; bottom: number; left: number }

    field: CustomFieldJson

    entries: EntryJson[]
};

export default function FloatRangeGraph({
    width, height, 
    margin = defaultMargin,
    field,
    entries = []
}: FloatRangeGraphProps) {
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
            let value = entry.custom_field_entries[field_id].value as FloatRange;

            if (min_y_domain > value.low) {
                min_y_domain = value.low;
            }

            if (max_y_domain < value.high) {
                max_y_domain = value.high;
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

    const y_axis_scale = scaleLinear<number>({
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
        <rect x={0} y={0} width={width} height={height} fill={background} rx={14} />
        <Group left={margin.left} top={margin.top}>
            <GridRows scale={y_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0" />
            <GridColumns scale={x_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0" />
            <line x1={xMax} x2={xMax} y1={0} y2={yMax} stroke="#e0e0e0" />
            <AxisBottom top={yMax} scale={x_axis_scale} numTicks={width > 520 ? 10 : 5} />
            <AxisLeft scale={y_axis_scale} />
            {data_groups.map(set => {
                return <Fragment key={Math.random()}>
                    <Threshold
                        id={`${Math.random()}`}
                        data={set}
                        x={d => x_axis_scale(getX(d))}
                        y0={d => y_axis_scale(getY0(d, field_id))}
                        y1={d => y_axis_scale(getY1(d, field_id))}
                        clipAboveTo={0}
                        clipBelowTo={yMax}
                        belowAreaProps={{
                            fill: 'green',
                            fillOpacity: 0.4,
                        }}
                    />
                    <DashedLinePath
                        data={set}
                        xGetter={d => x_axis_scale(getX(d))}
                        yGetter={d => y_axis_scale(getY0(d, field_id))}
                        marker={CircleMarker.url}
                    />
                    <SolidLinePath
                        data={set}
                        xGetter={d => x_axis_scale(getX(d))}
                        yGetter={d => y_axis_scale(getY1(d, field_id))}
                        marker={CircleMarker.url}
                    />
                </Fragment>
            })}
        </Group>
    </svg>
    );
}
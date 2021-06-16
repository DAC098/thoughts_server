import React, { Fragment, useMemo } from "react"
import { scaleTime, scaleLinear} from "@visx/scale"
import { AxisBottom, AxisLeft } from "@visx/axis"
import { Group } from '@visx/group'
import * as CurveType from '@visx/curve'
import { Threshold } from "@visx/threshold"
import { Line, Bar } from "@visx/shape"
import GridColumns from "@visx/grid/lib/grids/GridColumns"
import GridRows from "@visx/grid/lib/grids/GridRows"
import ParentSize from "@visx/responsive/lib/components/ParentSize"
import { bisector } from "d3-array"
import { CustomFieldJson, EntryJson } from "../../api/types"
import { useAppSelector } from "../../hooks/useApp"
import { common_ratios, containRatio } from "../../util/math"
import { entryIterator, EntryIteratorCB } from "../../components/graphs/util"
import * as CustomFieldEntryTypes from "../../api/custom_field_entry_types"
import * as CustomFieldTypes from "../../api/custom_field_types"
import { background } from "../../components/graphs/Float"
import { CircleMarker, TransCircleMarker } from "../../components/graphs/markers"
import { getDateZeroHMSM, timeToString } from "../../util/time"
import { defaultGetX } from "../../components/graphs/getters"
import { DashedLinePath, SolidLinePath } from "../../components/graphs/line_paths"

const entryIteratorInteger: EntryIteratorCB<CustomFieldEntryTypes.Integer> = (rtn, entry, field, value) => {
    if (rtn.min_y > value.value) {
        rtn.min_y = value.value;
    }

    if (rtn.max_y < value.value) {
        rtn.max_y = value.value;
    }
}

const entryIteratorIntegerRange: EntryIteratorCB<CustomFieldEntryTypes.IntegerRange> = (rtn, entry, field, value) => {
    if (rtn.min_y > value.low) {
        rtn.min_y = value.low;
    }

    if (rtn.max_y < value.high) {
        rtn.max_y = value.high;
    }
}

const entryIteratorFloat: EntryIteratorCB<CustomFieldEntryTypes.Float> = (rtn, entry, field, value) => {
    if (rtn.min_y > value.value) {
        rtn.min_y = value.value;
    }

    if (rtn.max_y < value.value) {
        rtn.max_y = value.value;
    }
}

const entryIteratorFloatRange: EntryIteratorCB<CustomFieldEntryTypes.FloatRange> = (rtn, entry, field, value) => {
    if (rtn.min_y > value.low) {
        rtn.min_y = value.low;
    }

    if (rtn.max_y < value.high) {
        rtn.max_y = value.high;
    }
}

const entryIteratorTime: EntryIteratorCB<CustomFieldEntryTypes.Time> = (rtn, entry, field, value) => {
    let time = new Date(value.value).getTime();

    if (rtn.min_y > time) {
        rtn.min_y = time;
    }

    if (rtn.max_y < time) {
        rtn.max_y = time;
    }
}

const entryIteratorTimeRange: EntryIteratorCB<CustomFieldEntryTypes.TimeRange> = (rtn, entry, field, value) => {
    let low = new Date(value.low).getTime();
    let high = new Date(value.high).getTime();

    if ((field.config as CustomFieldTypes.TimeRange).show_diff) {
        let diff = high - low;

        if (rtn.min_y > diff) {
            rtn.min_y = diff;
        }

        if (rtn.max_y < diff) {
            rtn.max_y = diff;
        }
    } else {
        if (rtn.min_y > low) {
            rtn.min_y = low;
        }

        if (rtn.max_y < high) {
            rtn.max_y = high;
        }
    }
}

function getEntryIterator(type: CustomFieldTypes.CustomFieldTypeName) {
    switch (type) {
        case CustomFieldTypes.CustomFieldTypeName.Integer:
            return entryIteratorInteger;
        case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
            return entryIteratorIntegerRange;
        case CustomFieldTypes.CustomFieldTypeName.Float:
            return entryIteratorFloat;
        case CustomFieldTypes.CustomFieldTypeName.FloatRange:
            return entryIteratorFloatRange;
        case CustomFieldTypes.CustomFieldTypeName.Time:
            return entryIteratorTime;
        case CustomFieldTypes.CustomFieldTypeName.TimeRange:
            return entryIteratorTimeRange;
    }
}

function getYScale(type: CustomFieldTypes.CustomFieldTypeName, min: number, max: number) {
    switch (type) {
        case CustomFieldTypes.CustomFieldTypeName.Time:
        case CustomFieldTypes.CustomFieldTypeName.TimeRange:
            return scaleTime<number>({
                domain: [min, max]
            });
        default:
            return scaleLinear<number>({
                domain: [min, max]
            });
    }
}

const bisectDate = bisector<EntryJson, Date>(entry => getDateZeroHMSM(entry.created)).left;

export interface GraphViewProps {
    field: CustomFieldJson
}

export const GraphView = ({field}: GraphViewProps) => {
    const margin = { top: 40, right: 30, bottom: 50, left: 80 };
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const entries_state = useAppSelector(state => state.entries);
    const tags_state = useAppSelector(state => state.tags);

    const loading_state = custom_fields_state.loading || entries_state.loading || tags_state.loading;

    const {
        min_x, min_y,
        max_x, max_y,
        data_groups,
        markers
    } = useMemo(() => {
        return entryIterator(
            entries_state.entries, 
            field,
            // @ts-ignore
            getEntryIterator(field.config.type)
        );
    }, [entries_state.key, field.config.type]);

    const y_axis_ticker_cb = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? (value, index) => {
                    return timeToString(typeof value === "number" ? value : value.valueOf(), false, true);
                } : null
            default:
                return null;
        }
    }, [field.config.type]);

    const get_y0_cb = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.Integer:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Integer).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Float:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Float).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Time:
                return (entry: EntryJson) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Time).value).getTime();
                }
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? (entry: EntryJson) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).high).getTime() - 
                           new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                } : (entry: EntryJson) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                }
        }
    }, [field.config.type]);

    const get_y1_cb = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).high;
                }
            case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                return (entry: EntryJson) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).high;
                }
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? null : (entry: EntryJson) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                }
        }
    }, [field.config.type]);

    const y_axis_scale = getYScale(field.config.type, min_y, max_y);
    const x_axis_scale = scaleTime<number>({domain: [min_x, max_x]});

    return <ParentSize 
        className="ms-StackItem"
        debounceTime={20}
        parentSizeStyles={{
            width: "auto", height: "400px",
            flexGrow: 1, flexShrink: 1,
            position: "relative",
            overflow: "hidden"
        }}
    >
        {({width: w, height: h}) => {
            if (loading_state)
                return null;
            
            const {width, height} = containRatio(w, h, common_ratios.r_16_9);
            const xMax = width - margin.left - margin.right;
            const yMax = height - margin.top - margin.bottom;

            y_axis_scale.range([yMax, 0]);
            x_axis_scale.range([0, xMax]);

            let content = [];

            switch (field.config.type) {
                case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                    content = data_groups.map(set => {
                        return <Fragment key={Math.random()}>
                            <Threshold
                                id={`${Math.random()}`}
                                data={set}
                                x={d => x_axis_scale(defaultGetX(d))}
                                y0={d => y_axis_scale(get_y0_cb(d))}
                                y1={d => y_axis_scale(get_y1_cb(d))}
                                clipAboveTo={0}
                                clipBelowTo={yMax}
                                belowAreaProps={{
                                    fill: 'green',
                                    fillOpacity: 0.4,
                                }}
                            />
                            <DashedLinePath
                                data={set}
                                xGetter={d => x_axis_scale(defaultGetX(d))}
                                yGetter={d => y_axis_scale(get_y0_cb(d))}
                                marker={TransCircleMarker.url}
                            />
                            <SolidLinePath
                                data={set}
                                xGetter={d => x_axis_scale(defaultGetX(d))}
                                yGetter={d => y_axis_scale(get_y1_cb(d))}
                                marker={CircleMarker.url}
                            />
                        </Fragment>
                    });
                    break;
                case CustomFieldTypes.CustomFieldTypeName.Integer:
                case CustomFieldTypes.CustomFieldTypeName.Float:
                    content = data_groups.map(set => {
                        return <Fragment key={Math.random()}>
                            <DashedLinePath
                                data={set}
                                xGetter={d => x_axis_scale(defaultGetX(d))}
                                yGetter={d => y_axis_scale(get_y0_cb(d))}
                                marker={TransCircleMarker.url}
                            />
                            <SolidLinePath
                                data={set}
                                curve={CurveType.curveBasis}
                                xGetter={d => x_axis_scale(defaultGetX(d))}
                                yGetter={d => y_axis_scale(get_y0_cb(d))}
                                marker={CircleMarker.url}
                            />
                        </Fragment>
                    });
                    break;
                case CustomFieldTypes.CustomFieldTypeName.Time:
                    content = data_groups.map(set => {
                        return <Fragment key={Math.random()}>
                            <SolidLinePath
                                data={set}
                                xGetter={d => x_axis_scale(defaultGetX(d))}
                                yGetter={d => y_axis_scale(get_y0_cb(d))}
                                marker={CircleMarker.url}
                            />
                        </Fragment>
                    });
                    break;
                case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                    if (!field.config.show_diff) {
                        content = data_groups.map(set => {
                            return <Fragment key={Math.random()}>
                                <Threshold
                                    id={`${Math.random()}`}
                                    data={set}
                                    x={d => x_axis_scale(defaultGetX(d))}
                                    y0={d => y_axis_scale(get_y0_cb(d))}
                                    y1={d => y_axis_scale(get_y1_cb(d))}
                                    clipAboveTo={0}
                                    clipBelowTo={yMax}
                                    belowAreaProps={{
                                        fill: 'green',
                                        fillOpacity: 0.4,
                                    }}
                                />
                                <DashedLinePath
                                    data={set}
                                    xGetter={d => x_axis_scale(defaultGetX(d))}
                                    yGetter={d => y_axis_scale(get_y0_cb(d))}
                                    marker={TransCircleMarker.url}
                                />
                                <SolidLinePath
                                    data={set}
                                    xGetter={d => x_axis_scale(defaultGetX(d))}
                                    yGetter={d => y_axis_scale(get_y1_cb(d))}
                                    marker={CircleMarker.url}
                                />
                            </Fragment>
                        });
                    } else {
                        content = data_groups.map((set) => {
                            return <Fragment key={Math.random()}>
                                <DashedLinePath
                                    data={set}
                                    xGetter={d => x_axis_scale(defaultGetX(d))}
                                    yGetter={d => y_axis_scale(get_y0_cb(d))}
                                    marker={TransCircleMarker.url}
                                />
                                <SolidLinePath
                                    data={set}
                                    curve={CurveType.curveBasis}
                                    xGetter={d => x_axis_scale(defaultGetX(d))}
                                    yGetter={d => y_axis_scale(get_y0_cb(d))}
                                    marker={CircleMarker.url}
                                />
                            </Fragment>
                        })
                    }
                    break;
            }

            return(
            <svg width={width} height={height}>
                <CircleMarker/>
                <TransCircleMarker/>
                <rect x={0} y={0} width={width} height={height} fill={background}/>
                <Bar
                    x={margin.left}
                    y={margin.top}
                    width={xMax}
                    height={yMax}
                    fill="transparent"
                    onMouseMove={(e) => {
                        
                    }}
                />
                <Group left={margin.left} top={margin.top}>
                    <GridRows scale={y_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
                    <GridColumns scale={x_axis_scale} width={xMax} height={yMax} stroke="#e0e0e0"/>
                    <line x1={xMax} x2={xMax} y1={0} y2={yMax} stroke="#e0e0e0"/>
                    <AxisBottom top={yMax} scale={x_axis_scale} numTicks={width > 520 ? 10 : 5}/>
                    <AxisLeft scale={y_axis_scale} tickFormat={y_axis_ticker_cb}/>
                    {content}
                    {markers.map((v, i) => {
                        let x_pos = x_axis_scale(v.day);

                        return <Fragment key={v.day}>
                            <Line
                                from={{x: x_pos, y: 0}}
                                to={{x: x_pos, y: yMax}}
                                stroke="#222"
                                strokeWidth={1.5}
                                strokeOpacity={0.8}
                                strokeDasharray="1,5"
                            />
                            <circle cx={x_pos} cy={0} r={2} fill="black"/>
                            <circle cx={x_pos} cy={yMax} r={2} fill="black"/>
                        </Fragment>
                    })}
                </Group>
            </svg>
            )
        }}
    </ParentSize>
}
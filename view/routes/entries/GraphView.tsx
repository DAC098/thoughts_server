import React, { Fragment, useMemo, useState } from "react"
import { scaleTime, scaleLinear} from "@visx/scale"
import { AxisBottom, AxisLeft } from "@visx/axis"
import { Group } from '@visx/group'
import * as CurveType from '@visx/curve'
import { Threshold } from "@visx/threshold"
import { Line, Bar } from "@visx/shape"
import { localPoint } from "@visx/event"
import { TooltipWithBounds } from "@visx/tooltip"
import GridColumns from "@visx/grid/lib/grids/GridColumns"
import GridRows from "@visx/grid/lib/grids/GridRows"
import ParentSize from "@visx/responsive/lib/components/ParentSize"
import { CustomField, ComposedEntry, Tag } from "../../apiv2/types"
import useAppSelector from "../../hooks/useAppSelector"
import { common_ratios, containRatio } from "../../util/math"
import { entryIterator, EntryIteratorCB } from "../../components/graphs/util"
import * as CustomFieldEntryTypes from "../../apiv2/custom_field_entry_types"
import * as CustomFieldTypes from "../../apiv2/custom_field_types"
import { background } from "../../components/graphs/Float"
import { CircleMarker, TransCircleMarker } from "../../components/graphs/markers"
import { dateFromUnixTime, getDateZeroHMSM, timeToString, zeroHMSM } from "../../util/time"
import { durationDays } from "../../util/duration"
import { defaultGetX } from "../../components/graphs/getters"
import { DashedLinePath, SolidLinePath } from "../../components/graphs/line_paths"
import { bisectorFind } from "../../util/search"
import { useHistory, useLocation } from "react-router-dom"
import { CustomFieldEntryCell } from "../../components/CustomFieldEntryCell"
import TagToken from "../../components/tags/TagItem"
import { Stack, Separator } from "@fluentui/react"
import { stringFromLocation } from "../../util/url"

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

interface TooltipDataProps {
    entry: ComposedEntry
    field: CustomField
    tags: {[key: string]: Tag}
}

const TooltipData = ({entry, field, tags}: TooltipDataProps) => {
    return <Stack>
        <div>{(dateFromUnixTime(entry.entry.day)).toLocaleDateString()}</div>
        {field.id in entry.custom_field_entries ?
            <>
                <Separator/>
                <CustomFieldEntryCell 
                    value={entry.custom_field_entries[field.id].value} 
                    config={field.config}
                />
            </>
            :
            null
        }
        {entry.markers.length > 0 ?
            <>
                <Separator/>
                <div>{entry.markers.map(v => v.title).join(" | ")}</div>
            </>
            :
            null
        }
        {entry.tags.length > 0 ?
            <>
                <Separator/>
                <div>{entry.tags.map(v => <TagToken
                    key={v}
                    color={tags[v].color}
                    title={tags[v].title}
                    fontSize={null}
                    lineHeight={20}
                />)}</div>
            </>
            :
            null
        }
    </Stack>
}

export interface GraphViewProps {
    field: CustomField
    user_specific: boolean
    owner: number
}

export const GraphView = ({field, user_specific, owner}: GraphViewProps) => {
    const margin = { top: 40, right: 30, bottom: 50, left: 80 };
    const custom_fields_state = useAppSelector(state => state.custom_fields);
    const entries_state = useAppSelector(state => state.entries);
    const tags_state = useAppSelector(state => state.tags);
    const location = useLocation();
    const history = useHistory();

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
    }, [field.id]);

    const get_y0_cb = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.Integer:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Integer).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Float:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Float).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Time:
                return (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Time).value).getTime();
                }
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).high).getTime() - 
                           new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                } : (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                }
        }
    }, [field.id]);

    const get_y1_cb = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).high;
                }
            case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).high;
                }
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? null : (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                }
        }
    }, [field.id]);

    const get_tooltip_y = useMemo(() => {
        switch (field.config.type) {
            case CustomFieldTypes.CustomFieldTypeName.Integer:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Integer).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.IntegerRange:
                return (entry: ComposedEntry) => {
                    return (((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).high -
                             (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).low) / 2) +
                             (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.IntegerRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Float:
                return (entry: ComposedEntry) => {
                    return (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Float).value;
                }
            case CustomFieldTypes.CustomFieldTypeName.FloatRange:
                return (entry: ComposedEntry) => {
                    return (((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).high -
                             (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).low) / 2) +
                             (entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.FloatRange).low;
                }
            case CustomFieldTypes.CustomFieldTypeName.Time:
                return (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.Time).value).getTime();
                }
            case CustomFieldTypes.CustomFieldTypeName.TimeRange:
                return field.config.show_diff ? (entry: ComposedEntry) => {
                    return new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).high).getTime() - 
                           new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                } : (entry: ComposedEntry) => {
                    let low_value = new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).low).getTime();
                    let high_value = new Date((entry.custom_field_entries[field.id].value as CustomFieldEntryTypes.TimeRange).high).getTime()
                    return ((high_value - low_value) / 2) + low_value;
                }
        }
    }, [field.id]);

    const y_axis_scale = getYScale(field.config.type, min_y, max_y);
    const x_axis_scale = scaleTime<number>({domain: [min_x, max_x]});

    return <ParentSize debounceTime={20}>
        {({width: w, height: h}) => {
            const [tooltip_index, setTooltipIndex] = useState(-1);

            if (loading_state)
                return null;
            
            const {width, height} = containRatio(w, h, common_ratios.r_16_9);
            const xMax = width - margin.left - margin.right;
            const yMax = height - margin.top - margin.bottom;

            y_axis_scale.range([yMax, 0]);
            x_axis_scale.range([0, xMax]);

            const handleToolTip = (event: React.TouchEvent<SVGRectElement> | React.MouseEvent<SVGRectElement>) => {
                const {x} = localPoint(event) || {x: 0};
                const x0 = x_axis_scale.invert(x - margin.left);
                let x_check = x0.getHours() > 12 ?
                    new Date(x0.getTime() + durationDays(1)) :
                    x0;

                zeroHMSM(x_check);

                let index = bisectorFind(entries_state.entries, x_check.getTime(), (f, v) => {
                    let v_day = dateFromUnixTime(v.entry.day);
                    zeroHMSM(v_day);
                    let v_time = v_day.getTime();

                    if (v_time === f) {
                        return 0;
                    } else if (v_time < f) {
                        return -1;
                    } else {
                        return 1;
                    }
                });

                if (tooltip_index !== index) {
                    setTooltipIndex(index);
                }
            }

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

            let tooltip_y = 0;
            let tooltip_x = 0;

            if (tooltip_index !== -1) {
                let day = dateFromUnixTime(entries_state.entries[tooltip_index].entry.day);
                zeroHMSM(day);
                tooltip_x = x_axis_scale(day);

                if (field.id in entries_state.entries[tooltip_index].custom_field_entries) {
                    tooltip_y = y_axis_scale(get_tooltip_y(entries_state.entries[tooltip_index]));
                }
            }

            return(<>
            <svg width={width} height={height}>
                <CircleMarker/>
                <TransCircleMarker/>
                <rect x={0} y={0} width={width} height={height} fill={background}/>
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
                    {tooltip_index !== -1 ?
                        <Fragment>
                            <Line
                                from={{x: tooltip_x, y: 0}}
                                to={{x: tooltip_x, y: yMax}}
                                stroke="#222"
                                strokeWidth={1.5}
                                strokeOpacity={0.8}
                                strokeDasharray="1,5"
                            />
                            <circle cx={tooltip_x} cy={0} r={2} fill="black"/>
                            <circle cx={tooltip_x} cy={yMax} r={2} fill="black"/>
                        </Fragment>
                        :
                        null
                    }
                    <Bar
                        width={xMax}
                        height={yMax}
                        fill="transparent"
                        onMouseMove={handleToolTip}
                        onMouseLeave={() => {
                            setTooltipIndex(-1);
                        }}
                        onClick={() => {
                            if (tooltip_index !== -1) {
                                history.push(
                                    `${location.pathname}/${entries_state.entries[tooltip_index].entry.id}?prev=${stringFromLocation(location)}`
                                );
                            }
                        }}
                    />
                </Group>
            </svg>
            {tooltip_index !== -1 ?
                <TooltipWithBounds top={tooltip_y} left={tooltip_x}>
                    <TooltipData 
                        entry={entries_state.entries[tooltip_index]} 
                        field={field}  
                        tags={tags_state.mapping}
                    />
                </TooltipWithBounds>
                :
                null
            }
            </>)
        }}
    </ParentSize>
}
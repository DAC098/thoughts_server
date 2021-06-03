import { Dropdown, MaskedTextField } from "@fluentui/react";
import React, { useEffect, useState } from "react";
import { get12hrStr, get24hrStr } from "../util/time";

const mask_types = {
    "hr_min": {
        mask: "99\:99",
        maskChar: "_",
        maskFormat: {
            "9": /[0-9]/
        }
    },
    "hr_min_sec": {
        mask: "99\:99\:99",
        maskChar: "_",
        maskFormat: {
            "9": /[0-9]/
        }
    }
};

interface TimeInputProps {
    value: Date

    AM_PM?: boolean
    include_seconds?: boolean

    onChange?: (date: Date) => void
}

const TimeInput = ({value, onChange, AM_PM = false, include_seconds = false}: TimeInputProps) => {
    const getValueStr = (date: Date) => {
        return AM_PM ? get12hrStr(date, false, include_seconds) : get24hrStr(date, include_seconds);
    };

    let [meridian, setMeridian] = useState(0);
    let [local_value_str, setLocalValueStr] = useState(getValueStr(value));
    let [err_msg, setErrMsg] = useState("");
    let [run_check, setRunCheck] = useState(false);

    const type = include_seconds ? mask_types["hr_min_sec"] : mask_types["hr_min"];

    const increment = () => {
        onChange?.(new Date(value.getTime() + (include_seconds ? 1 : 60) * 1000));
    }

    const decriment = () => {
        onChange?.(new Date(value.getTime() - (include_seconds ? 1 : 60) * 1000));
    }

    useEffect(() => {
        if (AM_PM) {
            setMeridian(value.getHours() >= 12 ? 1 : 0);
        }

        setLocalValueStr(getValueStr(value));
    }, [value, AM_PM, include_seconds]);

    return <MaskedTextField
        value={local_value_str}
        errorMessage={err_msg}
        styles={{
            "field": {width: include_seconds ? 70 : 55},
            "suffix": {padding: "0"}
        }}

        mask={type.mask}
        maskChar={type.maskChar}
        maskFormat={type.maskFormat}

        onBlur={e => {
            if (!run_check) {
                return;
            }

            let hr = parseInt(local_value_str.substring(0, 2));
            let min = parseInt(local_value_str.substring(3, 5));
            let sec = include_seconds ? parseInt(local_value_str.substring(6, 8)) : 0;

            if (isNaN(hr) || isNaN(min) || isNaN(sec)) {
                if (isNaN(hr)) {
                    setErrMsg("hours value is invalid");
                } else if (isNaN(min)) {
                    setErrMsg("minutes value is invalid");
                } else {
                    setErrMsg("seconds value is invalid");
                }

                return;
            }

            if (AM_PM) {
                if (hr >= 24) {
                    setErrMsg("hours cannot be greater than or equal to 12 or 24");
                    return;
                } else if (hr > 12) {
                    // they entered a 24 hr value
                } else if (hr == 12) {
                    if (meridian === 0) {
                        hr = 0;
                    }
                } else {
                    hr += (12 * meridian);
                }
            } else {
                if (hr >= 24) {
                    setErrMsg("hours cannot be greater than 24");
                    return;
                }
            }

            if (min >= 60) {
                setErrMsg("minutes cannot be greater than 60")
                return;
            }

            if (sec >= 60) {
                setErrMsg("seconds cannot be greater than 60");
                return;
            }
            
            let copy = new Date(value.getTime());
            copy.setHours(hr);
            copy.setMinutes(min);
            copy.setSeconds(sec);

            onChange?.(copy);
        }}
        onKeyDown={ev => {
            if (ev.key === "ArrowDown") {
                decriment();
            } else if (ev.key === "ArrowUp") {
                increment();
            }
        }}
        onChange={(e,v) => {
            setLocalValueStr(v);
            setErrMsg("");
            setRunCheck(true);
        }}

        onRenderSuffix={(props, defaultRender) => {
            if (AM_PM) {
                return <Dropdown
                    selectedKey={meridian}
                    options={[
                        {key: 0, text: "AM", selected: meridian === 0},
                        {key: 1, text: "PM", selected: meridian === 1}
                    ]}
                    onChange={(ev, item) => {
                        setMeridian(item.key as number);

                        if (err_msg.length === 0) {
                            let copy = new Date(value.getTime());
                            let hr = copy.getHours();

                            // know the meridian changed and that
                            // there is currently no error messages
                            // indicating bad input
                            if (item.key === 0) {
                                copy.setHours(hr - 12);
                            } else {
                                copy.setHours(hr + 12);
                            }

                            onChange?.(copy);
                        }
                    }}
                    styles={{
                        "title": {
                            backgroundColor: "rgba(0,0,0,0.0)",
                            border: 0
                        }
                    }}
                />
            } else {
                return null;
            }
        }}
    />
}

export default TimeInput 
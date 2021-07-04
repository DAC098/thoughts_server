import { year, month, week, day, hour, minute, second, millisecond } from "./duration"

export const max_date_number = Math.trunc(Number.MAX_SAFE_INTEGER / 1000);

export type DateType = string | number | Date;

function padISONum(number: number) {
    let normal = Math.floor(Math.abs(number));
    return (normal < 10 ? '0' : '') + normal;
}

export function toISOOffsetString(date: Date) {
    let tzo = -date.getTimezoneOffset();
    let dif = tzo >= 0 ? "+" : "-";
    
    return date.getFullYear() +
        '-' + padISONum(date.getMonth() + 1) +
        '-' + padISONum(date.getDate()) +
        'T' + padISONum(date.getHours()) +
        ':' + padISONum(date.getMinutes()) +
        ':' + padISONum(date.getSeconds()) +
        dif + padISONum(tzo / 60) +
        ':' + padISONum(tzo % 60);
}

export function compareDates(a: Date, b: Date) {
    return a.getTime() === b.getTime();
}

export function zeroHMSM(date: Date) {
    date.setHours(0);
    date.setMinutes(0);
    date.setSeconds(0);
    date.setMilliseconds(0);
}

export function getDateZeroHMSM(time: DateType) {
    let rtn = new Date(time);
    zeroHMSM(rtn);

    return rtn;
}

export function unixNow() {
    return Math.trunc(Date.now() / 1000)
}

export function unixTimeFromDate(date: Date) {
    return Math.trunc(date.getTime() / 1000)
}

export function unixTimeStrFromDate(date: Date) {
    return unixTimeFromDate(date).toString();
}

export function dateFromUnixTime(date: number) {
    if (!Number.isInteger(date) || date > max_date_number) {
        return null;
    }

    return new Date(Math.trunc(date * 1000));
}

export function dateFromUnixTimeZeroHMSM(date: number) {
    let rtn = dateFromUnixTime(date);

    if (rtn == null) {
        return null;
    }

    zeroHMSM(rtn);

    return rtn;
}

export function get24hrStr(date: Date, include_seconds: boolean = false) {
    return date.getHours().toString().padStart(2, "0") + ":" + date.getMinutes().toString().padStart(2, "0") + (include_seconds ? ":" + date.getSeconds().toString().padStart(2, "0") : "");
}

export function get12hrStr(date: Date, include_meridian: boolean = true, include_seconds: boolean = false) {
    let is_pm = date.getHours() >= 12;
    let hr = is_pm ? date.getHours() - 12 : date.getHours();
    return (hr === 0 ? "12" : hr.toString().padStart(2,"0")) + ":" + date.getMinutes().toString().padStart(2,"0") + (include_seconds ? ":" + date.getSeconds().toString().padStart(2, "0") : "") + (include_meridian ? " " + (is_pm ? "PM" : "AM") : "");
}

export function sameDate(lhs: Date, rhs: Date) {
    return lhs.getFullYear() === rhs.getFullYear() &&
           lhs.getMonth() === rhs.getMonth() &&
           lhs.getDate() === rhs.getDate();
}

export function displayDate(date: Date, as_24hr: boolean = true, include_seconds: boolean = false) {
    return `${date.toDateString()} ${as_24hr ? get24hrStr(date, include_seconds) : get12hrStr(date, true, include_seconds)}`
}

const diff_names = ["years", "months", "weeks", "days", "hours", "minutes", "seconds", "milliseconds"];
const diff_names_short = ["y", "m", "w", "d", "h", "min", "s", "ms"];
const diff_order = [year, month, week, day, hour, minute, second, millisecond];

export function timeToString(time: number, show_milli: boolean = true, short_hand: boolean = false): string {
    let working = time;
    let results = [];

    for (let i = 0; i < diff_order.length; ++i) {
        // critical section
        let value = Math.floor(working / diff_order[i]);
        working %= diff_order[i];

        results.push(value);
    }

    let str_list = [];

    for (let i = 0; i < results.length; ++i) {
        if (!show_milli && i === results.length - 1) {
            continue;
        }

        if (results[i] != 0) {
            str_list.push(`${results[i]} ${short_hand ? diff_names_short[i] : diff_names[i]}`);
        }
    }

    return str_list.join(" ");
}

/**
 * takes the difference between two dates and will display then as
 * years months days hours minutes seonds
 * @param lhs left hand side of operation
 * @param rhs right hand side of operation
 * @returns
 */
export function diffDates(lhs: Date, rhs: Date, show_milli: boolean = true, short_hand: boolean = false): string {
    // get the timestamps of both dates in milliseconds
    let diff = lhs.getTime() - rhs.getTime();

    return timeToString(diff, show_milli, short_hand);
}
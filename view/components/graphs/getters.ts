import { ComposedEntry } from "../../api/types";
import { dateFromUnixTime, getDateZeroHMSM, unixTimeFromDate, zeroHMSM } from "../../util/time";

export function defaultGetX(entry: ComposedEntry) {
    let day = dateFromUnixTime(entry.entry.day);
    zeroHMSM(day);
    
    return day.getTime();
}
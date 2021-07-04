import { EntryJson } from "../../api/types";
import { dateFromUnixTime, getDateZeroHMSM, unixTimeFromDate, zeroHMSM } from "../../util/time";

export function defaultGetX(entry: EntryJson) {
    let day = dateFromUnixTime(entry.day);
    zeroHMSM(day);
    
    return day.getTime();
}
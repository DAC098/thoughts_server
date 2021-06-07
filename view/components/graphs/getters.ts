import { EntryJson } from "../../api/types";
import { getDateZeroHMSM } from "../../util/time";

export function defaultGetX(entry: EntryJson) {
    return getDateZeroHMSM(entry.created).getTime();
}
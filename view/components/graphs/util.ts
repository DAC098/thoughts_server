import { CustomFieldEntryType } from "../../apiv2/custom_field_entry_types";
import { CustomField, ComposedEntry, EntryMarker } from "../../apiv2/types";
import { dateFromUnixTimeZeroHMSM, getDateZeroHMSM } from "../../util/time";

interface EntryMarkerInfo {
    day: number
    items: EntryMarker[]
}

export interface EntryIteratorResult {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
    data_groups: ComposedEntry[][]
    markers: EntryMarkerInfo[]
}

export type EntryIteratorCB<T extends CustomFieldEntryType> = (
    rtn: EntryIteratorResult,
    entry: ComposedEntry,
    field: CustomField,
    value: T
) => void;

export function entryIterator<T extends CustomFieldEntryType>(
    entries: ComposedEntry[],
    field: CustomField,
    cb: EntryIteratorCB<T>
): EntryIteratorResult {
    let field_id = field.id.toString();
    let field_entries: ComposedEntry[] = [];
    let rtn = {
        min_x: Infinity,
        max_x: -Infinity,
        min_y: Infinity,
        max_y: -Infinity,
        data_groups: [],
        markers: []
    };

    for (let entry of entries) {
        let date = dateFromUnixTimeZeroHMSM(entry.entry.day).getTime();

        if (rtn.min_x > date) {
            rtn.min_x = date;
        }

        if (rtn.max_x < date) {
            rtn.max_x = date;
        }

        if (entry.markers.length) {
            rtn.markers.push({
                day: date,
                items: entry.markers
            });
        }

        if (field_id in entry.custom_field_entries) {
            cb(rtn, entry, field, entry.custom_field_entries[field_id].value as T);

            field_entries.push(entry);
        } else {
            if (field_entries.length > 0) {
                rtn.data_groups.push(field_entries);
                field_entries = [];
            }
        }
    }

    if (field_entries.length > 0) {
        rtn.data_groups.push(field_entries);
        field_entries = [];
    }

    return rtn;
}
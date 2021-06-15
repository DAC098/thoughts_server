import { CustomFieldEntryType } from "../../api/custom_field_entry_types";
import { CustomFieldJson, EntryJson } from "../../api/types";
import { getDateZeroHMSM } from "../../util/time";

export interface EntryIteratorResult {
    min_x: number,
    max_x: number,
    min_y: number,
    max_y: number,
    data_groups: EntryJson[][]
}

export function entryIterator<T extends CustomFieldEntryType>(
    entries: EntryJson[], 
    field: CustomFieldJson,
    cb: (rtn: EntryIteratorResult, entry: EntryJson, field: CustomFieldJson, value: T) => void
): EntryIteratorResult {
    let field_id = field.id.toString();
    let field_entries: EntryJson[] = [];
    let rtn = {
        min_x: Infinity,
        max_x: -Infinity,
        min_y: Infinity,
        max_y: -Infinity,
        data_groups: []
    };

    for (let entry of entries) {
        let date = getDateZeroHMSM(entry.created).getTime();

        if (rtn.min_x > date) {
            rtn.min_x = date;
        }

        if (rtn.max_x < date) {
            rtn.max_x = date;
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
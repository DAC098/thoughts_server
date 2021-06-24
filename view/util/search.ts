type CheckResult = -1 | 0 | 1;

export function bisectorFind<T, U>(list: T[], find: U, check: (f: U, v: T) => CheckResult): number {
    let low = 0;
    let high = list.length;
    let fallback = 0;
    let prev_index = 0;

    while (low !== high && fallback <= list.length) {
        let index = Math.floor((high - low) / 2) + low;

        // in case the high and low are in a state
        // that will prevent them from updating
        if (prev_index === index) {
            return -1;
        }
        
        let result = check(find, list[index]);

        if (result === 0) {
            return index;
        } else if (result === 1) {
            low = index;
        } else {
            high = index;
        }

        ++fallback;
        prev_index = index;
    }

    if (fallback >= list.length) {
        console.warn("WARNING: bisectorFind: bisector fallback hit");
    }

    return -1;
}
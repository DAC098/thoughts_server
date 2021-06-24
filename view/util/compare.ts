export function compareNumbers(a?: number, b?: number) {
    if (a == null && b == null) {
        return 0;
    } else if (a == null) {
        return 1;
    } else if (b == null) {
        return -1;
    } else {
        if (a === b) {
            return 0;
        } else if (a > b) {
            return 1;
        } else {
            return -1;
        }
    }
}

export function compareBooleans(a?: boolean, b?: boolean) {
    if (a == null && b == null) {
        return 0;
    } else if (a == null) {
        return 1;
    } else if (b == null) {
        return -1;
    } else {
        if (a === b) {
            return 0;
        } else if (b) {
            return 1;
        } else {
            return -1;
        }
    }
}

export function compareStrings(a?: string, b?: string) {
    if (a == null && b == null) {
        return 0;
    } else if (a == null) {
        return 1;
    } else if (b == null) {
        return -1;
    } else {
        return a.localeCompare(b);
    }
}

export function compareDates(a?: Date, b?: Date) {
    if (a == null && b == null) {
        return 0;
    } else if (a == null) {
        return 1;
    } else if (b == null) {
        return -1;
    } else {
        let a_time = a.getTime();
        let b_time = b.getTime();

        if (a_time === b_time) {
            return 0;
        } else if (a_time < b_time) {
            return 1;
        } else {
            return -1;
        }
    }
}
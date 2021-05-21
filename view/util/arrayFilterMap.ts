type FilterCallback<T> = (value: T, index: number, array: T[]) => boolean;
type MapCallback<T, U> = (value: T, index: number, array: T[]) => U;

function arrayFilterMap<U = any,T = any>(array: T[], filter: FilterCallback<T>, map: MapCallback<T, U>): U[] {
    let rtn: U[] = [];

    for (let i = 0; i < array.length; ++i) {
        if (filter(array[i], i, array)) {
            rtn.push(map(array[i], i, array));
        }
    }

    return rtn;
}

export default arrayFilterMap;
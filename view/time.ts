export function getCreatedStringToDate(created: string) {
    let split = created.split("-");
    let year = parseInt(split[0]);
    let month = parseInt(split[1]) - 1;
    let day = parseInt(split[2]);

    return new Date(year, month, day);
}

export function getCreatedDateToString(created: Date) {
    let month = created.getMonth() + 1;
    let day = created.getDate();
    return `${created.getFullYear()}-${month < 10 ? `0${month}` : month}-${day < 10 ? `0${day}` : day}`;
}

export function compareDates(a: Date, b: Date) {
    return a.getTime() === b.getTime();
}
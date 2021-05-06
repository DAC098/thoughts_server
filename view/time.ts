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

export function unixNow() {
    return Math.trunc(Date.now() / 1000)
}

export function unixTime(date: Date) {
    return Math.trunc(date.getTime() / 1000)
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
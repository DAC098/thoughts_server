export function noOriginUrlString(url: URL | string) {
    if (typeof url === "string") {
        url = new URL(url);
    }

    return `${url.password}${url.search}${url.hash}`;
} 

export interface Location {
    pathname: string
    search: string
    hash: string
    origin?: string
}

export function urlFromLocation(obj: Location) {
    return new URL(location.pathname + location.search + location.hash, location.origin ?? window.location.origin);
}

export function stringFromLocation(obj: Location) {
    return location.pathname + location.search + location.hash;
}

export function currentUrl() {
    return urlFromLocation(window.location);
}
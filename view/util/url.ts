export function noOriginUrlString(url: URL | string) {
    if (typeof url === "string") {
        url = new URL(url);
    }

    return `${url.pathname}${url.search}${url.hash}`;
} 

export interface Options {
    encode?: boolean
    decode_search?: boolean
}

export interface Location {
    pathname: string
    search: string
    hash: string
    origin?: string
}

export function urlFromString(no_origin_url: string) {
    return new URL(no_origin_url, window.location.origin);
}

export function urlFromLocation(
    obj: Location,
    {encode = false, decode_search = false}: Options = {}
) {
    let search = decode_search ? decodeURIComponent(obj.search) : obj.search;

    return new URL(obj.pathname + search + obj.hash, obj.origin ?? window.location.origin);
}

export function stringFromLocation(
    obj: Location, 
    {encode = false, decode_search = false}: Options = {}
) {
    let search = decode_search ? decodeURIComponent(obj.search) : obj.search;
    let rtn = obj.pathname + search + obj.hash; 

    return encode ? encodeURIComponent(rtn) : rtn;
}

export function currentUrl() {
    return urlFromLocation(window.location);
}
import RequestError from "./error/RequestError";

export type Method = "GET" | "POST" | "PUT" | "DELETE";
export type Path = URL | string;

export function getNewRequest(method: Method, url: Path, headers: HeadersInit, body?: BodyInit) {
    if (typeof url === "string") {
        url = new URL(url, window.location.origin);
    }

    return new Request(url.toString(), {
        method,
        headers,
        body
    });
}

export interface ResponseJSON<T = any> {
    type?: string,
    message: string
    date: string,
    data?: T
}

export type JSONResponse<T> = T extends ResponseJSON ? T : ResponseJSON<T>;

async function handleResponse<T = any>(response: Response) : Promise<{body: JSONResponse<T>, response: Response, status: number}> {
    const content_type = response.headers.get("content-type");
    const body = content_type && content_type.includes("application/json") ?
        (await response.json() as JSONResponse<T>) :
        await response.text();

    if (typeof body === "string") {
        throw new RequestError(
            "UnexpectedContentType",
            "body of request was an unexpected type",
            response.status,
            response
        );
    }

    if (response.ok) {
        return {
            body, response,
            status: response.status
        }
    } else {
        let t = "unknown";
        let m = null;

        if (typeof body === "object") {
            t = body.type;
            m = body.message;
        }

        throw new RequestError(
            t, m, response.status, response
        );
    }
}

async function sendJSON<T = any>(method: Method, url: Path, body: any) {
    const request = getNewRequest(method, url, {
        "Accept": "application/json",
        "Content-Type": "application/json"
    }, JSON.stringify(body));
    const response = await fetch(request);

    return await handleResponse<T>(response);
}

export const json = {
    get: async<T = any> (url: Path) => {
        const request = getNewRequest("GET", url, {
            "Accept": "application/json"
        });
        const response = await fetch(request);
        
        return await handleResponse<T>(response);
    },
    post: <T = any> (url: Path, body: any) => sendJSON<T>("POST", url, body),
    put: <T = any> (url: Path, body: any) => sendJSON<T>("PUT", url, body),
    delete: <T = any> (url: Path, body?: any) => sendJSON<T>("DELETE", url, body)
}

window["request"] = {
    json
}
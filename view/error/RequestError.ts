import { cloneInteger, cloneString } from "../util/clone";

export default class RequestError extends Error {
    private status_: number;
    private request_: Response;
    private type_: string;

    constructor(type: string, message: string, status?: number, request?: Response) {
        super(message);
        this.type_ = type;
        this.status_ = status ?? 500;
        this.request_ = request;

        try {
            Error.captureStackTrace(this);
        } catch(e) {}
    }

    get status() {
        return this.status_;
    }

    get request() {
        return this.request_;
    }

    get type() {
        return this.type_;
    }

    public toJson() {
        return {
            error: "RequestError",
            status: cloneInteger(this.status_),
            type: cloneString(this.type_),
            message: cloneString(this.message)
        }
    }
}
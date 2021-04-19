import RequestError from "../RequestError";

export default class NotFound extends RequestError {
    constructor(message?: string, request?: Response) {
        super("NotFound", message ?? "not found", 404, request);

        Error.captureStackTrace(this);
    }
}
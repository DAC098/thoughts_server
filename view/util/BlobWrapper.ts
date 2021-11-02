export default class BlobWrapper {
    private blob: Blob;
    private url: string;

    public static fromBlob(blob: Blob) {
        let rtn = new BlobWrapper();
        rtn.blob = blob;
        
        return rtn;
    }

    public static fromParts(blob_parts?: BlobPart[], options?: BlobPropertyBag) {
        let rtn = new BlobWrapper();
        rtn.blob = new Blob(blob_parts, options);

        return rtn;
    }

    private constructor() {
        this.blob = null;
        this.url = null;
    }

    private createUrl() {
        if (this.url == null) {
            this.url = URL.createObjectURL(this.blob);
        }
    }

    public deleteUrl() {
        if (this.url != null) {
            URL.revokeObjectURL(this.url);

            return true;
        } else {
            return false;
        }
    }

    public getUrl() {
        this.createUrl();

        return this.url;
    }

    public getBlob() {
        return this.blob;
    }
}
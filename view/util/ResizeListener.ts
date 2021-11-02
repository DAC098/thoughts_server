import { Emitter, BuiltInEvents, EventKey, EventReceiver } from "./events"

type Events = {
    resize: [width: number, height: number]
};

class ResizeListener extends Emitter<BuiltInEvents<Events>> {

    private frame_id: number = null;

    constructor() {
        super();

        this.on("newListener", this.onNewListener);
    }

    onResize = () => {
        if (this.frame_id == null) {
            this.frame_id = window.requestAnimationFrame(() => {
                this.emit("resize", window.innerWidth, window.innerHeight);
                this.frame_id = null;
            });
        }
    }

    onNewListener = <K extends EventKey<Events>>(event: K, listener: EventReceiver<Events[K]>) => {
        if (event === "resize" && this.listenerCount("resize") === 0) {
            window.addEventListener("resize", this.onResize);
        }
    }

    onRemoveListener = () => {
        if (this.listenerCount("resize") === 0) {
            window.removeEventListener("resize", this.onResize);
        }
    }
}

const resize_listener = new ResizeListener();

export default resize_listener;
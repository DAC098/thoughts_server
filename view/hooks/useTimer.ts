import { useEffect, useRef } from "react"

export default function useTimer(callback: () => void) {
    let timer_ref = useRef<number>();
    let cb_ref = useRef(callback);

    useEffect(() => {
        cb_ref.current = callback;
    }, [callback]);

    return {
        set: (ms: number) => {
            if (timer_ref.current)
                window.clearTimeout(timer_ref.current);

            timer_ref.current = window.setTimeout(() => {
                window.clearTimeout(timer_ref.current);
                cb_ref.current();
            }, ms);
        },
        clear: () => {
            if (timer_ref.current)
                window.clearTimeout(timer_ref.current);
        }
    }
}

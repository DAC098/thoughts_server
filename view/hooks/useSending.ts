import { useState } from "react"

export default function useSending<T extends any[]>(cb: (...args: T) => Promise<void>): [boolean, (...args: T) => void] {
    let [sending, setSending] = useState(false);

    return [sending, (...pass: T) => {
        setSending(true);

        cb(...pass).then(() => setSending(false))
    }]
}
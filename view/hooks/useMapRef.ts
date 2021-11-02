import { useRef } from "react";

export default function useMapRef<K,V>() {
    let map_ref = useRef<Map<K, V>>(new Map());

    return {
        get: (key: K) => {
            return map_ref.current.get(key);
        },
        set: (key: K, value: V) => {
            return map_ref.current.set(key, value);
        },
        keys: () => {
            return map_ref.current.keys();
        },
        values: () => {
            return map_ref.current.values();
        },
        entries: () => {
            return map_ref.current.entries();
        }
    }
}
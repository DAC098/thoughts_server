import { EventEmitter } from "events"

export type EventMap = {[key: string]: any[]};
export type EventKey<T extends EventMap> = string & keyof T;
export type EventReceiver<T extends any[]> = (...args: T) => void;

export type TypedBuiltInEvents<T extends EventMap, K extends EventKey<T>> = {
    newListener: [eventName: K, listener: EventReceiver<T[K]>],
    removeListener: [eventName: K, listener: EventReceiver<T[K]>]
}

type Except<ObjectType, KeysType extends keyof ObjectType> = Pick<ObjectType, Exclude<keyof ObjectType, KeysType>>;
type Simplify<T> = {[KeyType in keyof T]: T[KeyType]};
type Merge_<FirstType, SecondType> = Except<FirstType, Extract<keyof FirstType, keyof SecondType>> & SecondType;
type Merge<FirstType, SecondType> = Simplify<Merge_<FirstType, SecondType>>;

export type BuiltInEvents<T extends EventMap> = Merge<{
    newListener: [eventName: keyof T, listener: EventReceiver<T[keyof T]>],
    removeListener: [eventName: keyof T, listener: EventReceiver<T[keyof T]>]
}, T>;

export interface Emitter<T extends EventMap> extends EventEmitter {
    on<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    once<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    addListener<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    prependListener<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    prependOnceListener<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this

    off<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    removeListener<K extends EventKey<T>>(event: K, listener: EventReceiver<T[K]>): this
    removeAllListeners<K extends EventKey<T>>(event: K): this

    emit<K extends EventKey<T>>(event: K, ...args: T[K]): boolean

    listeners<K extends EventKey<T>>(event: K): EventReceiver<T[K]>[]
    rawListeners<K extends EventKey<T>>(event: K): EventReceiver<T[K]>[]
    listenerCount<K extends EventKey<T>>(event: K): number

    eventNames(): EventKey<T>[];
}

export class Emitter<T extends EventMap> extends EventEmitter {}
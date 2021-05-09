export type SliceActionTypes<T> = {
    [K in keyof T]: T[K] extends (...args: any) => infer A ? A : any
}[keyof T];
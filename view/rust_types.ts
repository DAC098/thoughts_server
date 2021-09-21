export type u8 = number;
export type u16 = number;
export type u32 = number;
export type u64 = number;
export type u128 = number;

export type i8 = number;
export type i16 = number;
export type i32 = number;
export type i64 = number;
export type i128 = number;

export type String = string;
export type bool = boolean;

export type Option<T> = T | null;
export type Vec<T> = T[];
export type HashMap<K extends string | number | symbol, T> = {[P in K]: T}
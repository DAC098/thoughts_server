export function cloneString(string: string): string {
    return string.slice();
}

export function cloneInteger(integer: number): number {
    return integer + 0;
}

export function cloneFloat(float: number): number {
    return float + 0.0;
}

export function cloneBoolean(boolean: boolean): boolean {
    return !!boolean;
}

export function optionalCloneString(string?: string): string|null {
    return string?.slice() ?? null;
}

export function optionalCloneInteger(integer?: number): number|null {
    return integer != null ? cloneInteger(integer) : null;
}

export function optionalCloneBoolean(boolean?: boolean): boolean|null {
    return boolean != null ? cloneBoolean(boolean) : null;
}
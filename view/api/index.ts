export function getURL(url: string) {
    return new URL(url, window.location.origin);
}

export * as entries from "./entries"
export * as mood_fields from "./mood_fields"
export * as users from "./users"
export * as login from "./login"
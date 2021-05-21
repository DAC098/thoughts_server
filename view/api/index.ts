export function getURL(url: string) {
    return new URL(url, window.location.origin);
}

import * as entries from "./entries"
import * as mood_fields from "./mood_fields"
import * as users from "./users"
import * as login from "./login"
import * as admin from "./admin"
import * as tags from "./tags"

const api = {
    entries,
    mood_fields,
    users,
    login,
    admin,
    tags
};

export default api;
export const allow_media_devices = !!window.navigator.mediaDevices;
export const allow_get_user_media = allow_media_devices && window.navigator.mediaDevices.getUserMedia;
export const allow_enumerate_devices = allow_media_devices && window.navigator.mediaDevices.enumerateDevices;

export async function getUserMedia(constraints?: MediaStreamConstraints) {
    if (allow_get_user_media) {
        return null;
    }

    return navigator.mediaDevices.getUserMedia(constraints);
}

export async function enumerateDevices() {
    if (allow_enumerate_devices) {
        return null;
    }

    return navigator.mediaDevices.enumerateDevices();
}
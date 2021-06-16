export function rand(max = 100_000) {
    return Math.trunc(Math.random() * max);
}

export function randHex(max = 100_000) {
    return rand(max).toString(16);
}

export function randCrypto() {
    let array = new Uint32Array(1);
    crypto.getRandomValues(array);
    return array[0];
}

export function randCryptoHex() {
    return randCrypto().toString(16);
}
export const min_brightness = 65;

export function getBrightness(hex_color: string): number {
    let rgb = parseInt(hex_color.substring(1), 16);
    let r = (rgb >> 16) & 0xff;
    let g = (rgb >>  8) & 0xff;
    let b = (rgb >>  0) & 0xff;

    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
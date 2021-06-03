type Ratio = [upper: number, lower: number];

export const common_ratios: {[key: string]: Ratio} = {
    r_16_10: [16, 10],
    r_16_9: [16, 9],
    r_14_9: [14, 9],
    r_11_8: [11, 8],
    r_9_16: [9, 16],
    r_6_5: [6, 5],
    r_5_4: [5, 4],
    r_5_3: [5, 3],
    r_4_3: [4, 3],
    r_3_2: [3, 2],
    r_2_1: [2, 1]
}

export function calcWidthRatio(height: number, ratio: Ratio): number {
    return (ratio[0] / ratio[1]) * height;
}

export function calcHeightRatio(width: number, ratio: Ratio): number {
    return (ratio[1] / ratio[0]) * width;
}

export function containRatio(width: number, height: number, ratio?: Ratio): {width: number, height: number} {
    if (ratio == null) {
        ratio = [width / height, 1];
    }
    
    let ratio_height = calcHeightRatio(width, ratio);
    let ratio_width = calcWidthRatio(height, ratio);
    let favor_width = width < ratio_width;

    return {
        width: favor_width ? width : ratio_width,
        height: favor_width ? ratio_height : height
    }
}
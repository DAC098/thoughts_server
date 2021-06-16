import React from "react"
import { MarkerCircle } from "@visx/marker"

const CircleMarkerId = "marker-circle";

export function CircleMarker() {
    return <MarkerCircle 
        id={CircleMarkerId} 
        fill="#333" 
        size={2} refX={2}
        onMouseEnter={(e) => {
            console.log(e);
        }}
    />
}

CircleMarker.id = CircleMarkerId;
CircleMarker.url = `url(#${CircleMarkerId})`;

const TransCircleMarkerId = "marker-trans-circle";

export const TransCircleMarker = () => {
    return <MarkerCircle 
        id={TransCircleMarkerId}  
        stroke="rgba(33,33,33,0.5)" 
        fill="transparent" 
        size={3} refX={3}
        onMouseEnter={(e) => {
            console.log(e);
        }}
    />
}

TransCircleMarker.id = TransCircleMarkerId;
TransCircleMarker.url = `url(#${TransCircleMarkerId})`;

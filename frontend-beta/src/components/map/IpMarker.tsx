"use client";

import { CircleMarker, Tooltip } from "react-leaflet";
import type { LeafletEventHandlerFnMap } from "leaflet";
import type { GeoMapPoint } from "@/lib/types";

interface IpMarkerProps {
	point: GeoMapPoint;
	isSelected: boolean;
	onClick: () => void;
	clusterCount?: number;
}

const riskColor: Record<GeoMapPoint["risk"], string> = {
	critical: "#ff5555",
	high: "#bd93f9",
	medium: "#ffb86c",
	low: "#50fa7b",
};

export default function IpMarker({
	point,
	isSelected,
	onClick,
	clusterCount,
}: IpMarkerProps) {
	const isCluster = (clusterCount ?? 1) > 1;
	const baseColor = riskColor[point.risk];
	const radius = isCluster ? Math.min(18, 8 + Math.log2(clusterCount ?? 1) * 2.5) : isSelected ? 8 : 6;
	const handlers: LeafletEventHandlerFnMap = { click: onClick };

	return (
		<CircleMarker
			center={[point.lat, point.lon]}
			radius={radius}
			pathOptions={{
				color: "#0b0f18",
				weight: isSelected ? 2 : 1,
				fillColor: baseColor,
				fillOpacity: isCluster ? 0.85 : 0.95,
			}}
			eventHandlers={handlers}
		>
			<Tooltip direction="top" offset={[0, -6]} opacity={1}>
				<div className="text-[11px] font-medium">
					{isCluster ? `${clusterCount} IPs` : point.ip}
					<div className="text-[10px] opacity-80">
						{point.city ? `${point.city}, ` : ""}
						{point.country}
					</div>
				</div>
			</Tooltip>
		</CircleMarker>
	);
}

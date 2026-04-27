"use client";

import dynamic from "next/dynamic";
import type { GeoMapPoint, HopTimelinePoint } from "@/lib/types";

const WorldMapCanvas = dynamic(() => import("@/components/map/WorldMapCanvas"), {
	ssr: false,
	loading: () => (
		<div className="h-full w-full rounded-xl bg-surface-container-lowest/70 flex items-center justify-center">
			<p className="text-xs text-outline animate-pulse tracking-widest uppercase font-bold">
				Loading map engine...
			</p>
		</div>
	),
});

interface WorldMapProps {
	points: GeoMapPoint[];
	selectedId: string | null;
	onSelect: (point: GeoMapPoint) => void;
	hopPath: HopTimelinePoint[];
	activeHop: number;
}

export default function WorldMap({ points, selectedId, onSelect, hopPath, activeHop }: WorldMapProps) {
	return (
		<WorldMapCanvas
			points={points}
			selectedId={selectedId}
			onSelect={onSelect}
			hopPath={hopPath}
			activeHop={activeHop}
		/>
	);
}

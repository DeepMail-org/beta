"use client";

import { useMemo } from "react";
import { MapContainer, TileLayer, ZoomControl, useMap, Polyline } from "react-leaflet";
import { useEffect } from "react";
import type { GeoMapPoint, HopTimelinePoint } from "@/lib/types";
import IpMarker from "@/components/map/IpMarker";

interface WorldMapCanvasProps {
	points: GeoMapPoint[];
	selectedId: string | null;
	onSelect: (point: GeoMapPoint) => void;
	hopPath: HopTimelinePoint[];
	activeHop: number;
}

interface ClusterPoint {
	key: string;
	lat: number;
	lon: number;
	count: number;
	representative: GeoMapPoint;
	items: GeoMapPoint[];
}

const riskOrder: Record<GeoMapPoint["risk"], number> = {
	critical: 4,
	high: 3,
	medium: 2,
	low: 1,
};

function clusterPoints(points: GeoMapPoint[]): ClusterPoint[] {
	if (points.length <= 100) {
		return points.map((point) => ({
			key: point.id,
			lat: point.lat,
			lon: point.lon,
			count: 1,
			representative: point,
			items: [point],
		}));
	}

	const buckets = new Map<string, ClusterPoint>();
	for (const point of points) {
		const latBucket = Math.round(point.lat * 2) / 2;
		const lonBucket = Math.round(point.lon * 2) / 2;
		const key = `${latBucket}:${lonBucket}`;

		const existing = buckets.get(key);
		if (!existing) {
			buckets.set(key, {
				key,
				lat: latBucket,
				lon: lonBucket,
				count: 1,
				representative: point,
				items: [point],
			});
			continue;
		}

		existing.count += 1;
		existing.items.push(point);
		if (riskOrder[point.risk] > riskOrder[existing.representative.risk]) {
			existing.representative = point;
		}
	}

	return Array.from(buckets.values());
}

function FocusOnSelect({ selected }: { selected: GeoMapPoint | null }) {
	const map = useMap();

	useEffect(() => {
		if (!selected) return;
		const currentZoom = map.getZoom();
		map.flyTo([selected.lat, selected.lon], Math.max(currentZoom, 4), {
			duration: 0.6,
		});
	}, [selected, map]);

	return null;
}

export default function WorldMapCanvas({
	points,
	selectedId,
	onSelect,
	hopPath,
	activeHop,
}: WorldMapCanvasProps) {
	const clusters = useMemo(() => clusterPoints(points), [points]);
	const selected = points.find((point) => point.id === selectedId) ?? null;

	const ipToPoint = useMemo(() => {
		const map = new Map<string, GeoMapPoint>();
		for (const point of points) {
			map.set(point.ip, point);
		}
		return map;
	}, [points]);

	const hopCoordinates = useMemo(() => {
		const coords: [number, number][] = [];
		for (const hop of hopPath) {
			if (!hop.ip) continue;
			const point = ipToPoint.get(hop.ip);
			if (!point) continue;
			coords.push([point.lat, point.lon]);
		}
		return coords;
	}, [hopPath, ipToPoint]);

	const activeHopCoords = useMemo(() => {
		if (activeHop < 0 || hopCoordinates.length === 0) return [] as [number, number][];
		return hopCoordinates.slice(0, Math.min(activeHop + 1, hopCoordinates.length));
	}, [activeHop, hopCoordinates]);

	return (
		<MapContainer
			center={[20, 0]}
			zoom={2}
			minZoom={2}
			maxZoom={12}
			scrollWheelZoom
			doubleClickZoom
			touchZoom
			zoomControl={false}
			worldCopyJump
			style={{ width: "100%", height: "100%", borderRadius: "0.9rem" }}
			className="threat-map"
		>
			<TileLayer
				attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> &copy; CARTO'
				url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
				subdomains={["a", "b", "c", "d"]}
			/>
			<ZoomControl position="topright" />

			{hopCoordinates.length > 1 && (
				<Polyline positions={hopCoordinates} pathOptions={{ color: "#6272a4", weight: 2, opacity: 0.45 }} />
			)}
			{activeHopCoords.length > 1 && (
				<Polyline
					positions={activeHopCoords}
					pathOptions={{ color: "#8be9fd", weight: 3, opacity: 0.9, dashArray: "8 6" }}
				/>
			)}

			{clusters.map((cluster) => {
				if (cluster.count > 1) {
					const clusteredPoint: GeoMapPoint = {
						...cluster.representative,
						id: `cluster-${cluster.key}`,
						ip: `${cluster.count} IPs`,
						lat: cluster.lat,
						lon: cluster.lon,
					};

					return (
						<IpMarker
							key={cluster.key}
							point={clusteredPoint}
							clusterCount={cluster.count}
							isSelected={false}
							onClick={() => onSelect(cluster.representative)}
						/>
					);
				}

				return (
					<IpMarker
						key={cluster.representative.id}
						point={cluster.representative}
						isSelected={selectedId === cluster.representative.id}
						onClick={() => onSelect(cluster.representative)}
					/>
				);
			})}

			<FocusOnSelect selected={selected} />
		</MapContainer>
	);
}

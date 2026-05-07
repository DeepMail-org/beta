import { MapContainer, TileLayer, CircleMarker, Tooltip } from "react-leaflet";
import type { Hotspot } from "./interactive-map";

export function MapComponent({ hotspots }: { hotspots: Hotspot[] }) {
  // Use CartoDB Dark Matter tile layer for a dark theme aesthetic
  return (
    <MapContainer
      center={[20, 0]}
      zoom={2}
      minZoom={1.5}
      scrollWheelZoom={false}
      style={{ width: "100%", height: "100%", background: "#000000" }}
      className="z-0"
    >
      <TileLayer
        attribution='&copy; <a href="https://carto.com/attributions">CARTO</a>'
        url="https://{s}.basemaps.cartocdn.com/dark_all/{z}/{x}/{y}{r}.png"
      />
      
      {hotspots.map((spot, i) => (
        <CircleMarker
          key={i}
          center={[spot.lat, spot.lng]}
          pathOptions={{
            color: "var(--color-danger)",
            fillColor: "var(--color-danger)",
            fillOpacity: spot.intensity,
            weight: 1,
          }}
          radius={Math.max(4, spot.intensity * 12)}
        >
          <Tooltip direction="top" offset={[0, -10]} opacity={1}>
            <span className="font-semibold">{spot.label}</span>
            <br />
            Intensity: {(spot.intensity * 100).toFixed(0)}%
          </Tooltip>
        </CircleMarker>
      ))}
    </MapContainer>
  );
}

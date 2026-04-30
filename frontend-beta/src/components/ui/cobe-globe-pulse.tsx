"use client";

import { useEffect, useRef, useCallback, type PointerEvent as ReactPointerEvent } from "react";
import createGlobe from "cobe";

interface PulseMarker {
	id: string;
	location: [number, number];
	delay: number;
}

interface GlobePulseProps {
	markers?: PulseMarker[];
	className?: string;
	speed?: number;
	/** RGB triplet 0..1 for the dot colour of land */
	baseColor?: [number, number, number];
	/** RGB triplet 0..1 for the marker colour */
	markerColor?: [number, number, number];
	/** RGB triplet 0..1 for the rim glow */
	glowColor?: [number, number, number];
	/** Brightness multiplier for the dotted map (1..15). Higher = more vivid */
	mapBrightness?: number;
	/** Whether the user can rotate the globe with their pointer (default true) */
	interactive?: boolean;
}

const defaultMarkers: PulseMarker[] = [
	{ id: "pulse-1", location: [51.51, -0.13], delay: 0 },
	{ id: "pulse-2", location: [40.71, -74.01], delay: 0.5 },
	{ id: "pulse-3", location: [35.68, 139.65], delay: 1 },
	{ id: "pulse-4", location: [-33.87, 151.21], delay: 1.5 },
];

export function GlobePulse({
	markers = defaultMarkers,
	className = "",
	speed = 0.003,
	baseColor = [0.18, 0.42, 0.95],
	markerColor = [0.45, 0.78, 1.0],
	glowColor = [0.18, 0.42, 0.95],
	mapBrightness = 12,
	interactive = true,
}: GlobePulseProps) {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const pointerInteracting = useRef<{ x: number; y: number } | null>(null);
	const dragOffset = useRef({ phi: 0, theta: 0 });
	const phiOffsetRef = useRef(0);
	const thetaOffsetRef = useRef(0);
	const isPausedRef = useRef(false);

	const handlePointerDown = useCallback((e: ReactPointerEvent) => {
		pointerInteracting.current = { x: e.clientX, y: e.clientY };
		if (canvasRef.current) canvasRef.current.style.cursor = "grabbing";
		isPausedRef.current = true;
	}, []);

	const handlePointerUp = useCallback(() => {
		if (pointerInteracting.current !== null) {
			phiOffsetRef.current += dragOffset.current.phi;
			thetaOffsetRef.current += dragOffset.current.theta;
			dragOffset.current = { phi: 0, theta: 0 };
		}
		pointerInteracting.current = null;
		if (canvasRef.current) canvasRef.current.style.cursor = "grab";
		isPausedRef.current = false;
	}, []);

	useEffect(() => {
		const handlePointerMove = (e: globalThis.PointerEvent) => {
			if (pointerInteracting.current !== null) {
				dragOffset.current = {
					phi: (e.clientX - pointerInteracting.current.x) / 300,
					theta: (e.clientY - pointerInteracting.current.y) / 1000,
				};
			}
		};
		window.addEventListener("pointermove", handlePointerMove, { passive: true });
		window.addEventListener("pointerup", handlePointerUp, { passive: true });
		return () => {
			window.removeEventListener("pointermove", handlePointerMove);
			window.removeEventListener("pointerup", handlePointerUp);
		};
	}, [handlePointerUp]);

	useEffect(() => {
		if (!canvasRef.current) return;
		const canvas = canvasRef.current;
		let globe: ReturnType<typeof createGlobe> | null = null;
		let animationId: number;
		let phi = 0;

		function init() {
			const width = canvas.offsetWidth;
			if (width === 0 || globe) return;

			globe = createGlobe(canvas, {
				devicePixelRatio: Math.min(window.devicePixelRatio || 1, 2),
				width,
				height: width,
				phi: 0,
				theta: 0.2,
				dark: 1,
				diffuse: 1.4,
				mapSamples: 16000,
				mapBrightness,
				baseColor,
				markerColor,
				glowColor,
				markers: markers.map((m) => ({
					location: m.location,
					size: 0.03,
				})),
			} as Parameters<typeof createGlobe>[1]);

			const animateFrame = () => {
				if (!isPausedRef.current) phi += speed;
				globe!.update({
					phi: phi + phiOffsetRef.current + dragOffset.current.phi,
					theta: 0.2 + thetaOffsetRef.current + dragOffset.current.theta,
				});
				animationId = requestAnimationFrame(animateFrame);
			};
			animateFrame();
			setTimeout(() => canvas && (canvas.style.opacity = "1"));
		}

		if (canvas.offsetWidth > 0) {
			init();
		} else {
			const ro = new ResizeObserver((entries) => {
				if (entries[0]?.contentRect.width > 0) {
					ro.disconnect();
					init();
				}
			});
			ro.observe(canvas);
		}

		return () => {
			if (animationId) cancelAnimationFrame(animationId);
			if (globe) globe.destroy();
		};
	}, [markers, speed]);

	return (
		<div className={`relative aspect-square select-none ${className}`}>
			<style>{`
				@keyframes pulse-expand {
					0% { transform: scaleX(0.3) scaleY(0.3); opacity: 0.8; }
					100% { transform: scaleX(1.5) scaleY(1.5); opacity: 0; }
				}
			`}</style>
			<canvas
				ref={canvasRef}
				onPointerDown={interactive ? handlePointerDown : undefined}
				style={{
					width: "100%",
					height: "100%",
					cursor: interactive ? "grab" : "default",
					opacity: 0,
					transition: "opacity 1.2s ease",
					borderRadius: "50%",
					touchAction: "none",
					pointerEvents: interactive ? "auto" : "none",
				}}
			/>
		</div>
	);
}

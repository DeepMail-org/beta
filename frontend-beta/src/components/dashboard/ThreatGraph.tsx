"use client";

import {
	useEffect,
	useRef,
	useState,
	useMemo,
	forwardRef,
	useImperativeHandle,
} from "react";
import ForceGraph2D, { ForceGraphMethods } from "react-force-graph-2d";
import { GraphNode, GraphLink } from "@/hooks/useThreatGraph";
import { motion, AnimatePresence } from "motion/react";

export interface ThreatGraphHandle {
	zoomIn: () => void;
	zoomOut: () => void;
	center: () => void;
}

interface ThreatGraphProps {
	graphData?: { nodes: GraphNode[]; links: GraphLink[] };
	geoPoints?: any[];
	onNodeClick?: (node: GraphNode) => void;
	width?: number;
	height?: number;
	showLabels?: boolean;
}

const ThreatGraph = forwardRef<ThreatGraphHandle, ThreatGraphProps>(
	(
		{
			graphData: propGraphData,
			geoPoints,
			onNodeClick,
			width = 600,
			height = 600,
			showLabels = true,
		},
		ref,
	) => {
		const fgRef = useRef<any>(null);
		const [hoveredNode, setHoveredNode] = useState<GraphNode | null>(null);

		useImperativeHandle(ref, () => ({
			zoomIn: () => {
				const currentZoom = fgRef.current?.zoom();
				fgRef.current?.zoom(currentZoom * 1.5, 400);
			},
			zoomOut: () => {
				const currentZoom = fgRef.current?.zoom();
				fgRef.current?.zoom(currentZoom * 0.7, 400);
			},
			center: () => {
				fgRef.current?.zoomToFit(800, 100);
				fgRef.current?.centerAt(0, 0, 400);
			},
		}));

		const graphData = useMemo(() => {
			if (propGraphData) return propGraphData;
			if (geoPoints) {
				const nodes: GraphNode[] = geoPoints.map((p) => ({
					id: p.id,
					type: "ip",
					value: p.value,
					risk: p.risk,
					isMalicious: p.risk === "Critical",
					metadata: p,
				}));
				nodes.push({
					id: "core",
					type: "email",
					value: "Threat Feed",
					risk: "Safe",
					isMalicious: false,
				});
				const links: GraphLink[] = nodes
					.filter((n) => n.id !== "core")
					.map((n) => ({
						source: "core",
						target: n.id,
						type: "contains",
					}));
				return { nodes, links };
			}
			return { nodes: [], links: [] };
		}, [propGraphData, geoPoints]);

		// Physics tuning for investigation scale
		useEffect(() => {
			if (fgRef.current) {
				fgRef.current.d3Force("charge")?.strength(-250);
				fgRef.current.d3Force("link")?.distance(70);
				fgRef.current.d3Force("center")?.strength(0.5);
			}
		}, []);

		// Auto-zoom to fit nodes on data update and initial load
		useEffect(() => {
			const timer = setTimeout(() => {
				if (fgRef.current && (graphData.nodes?.length || 0) > 0) {
					// Use smaller padding for small containers (dashboard)
					const padding = Math.max(
						20,
						Math.min(width, height) * 0.15,
					);
					fgRef.current.zoomToFit(800, padding);
					fgRef.current.centerAt(0, 0, 400);
				}
			}, 300);
			return () => clearTimeout(timer);
		}, [graphData.nodes?.length, width, height]);

		const nodeColors: Record<string, string> = {
			email: "#bd93f9", // Dracula Purple
			ip: "#8be9fd", // Dracula Cyan
			domain: "#50fa7b", // Dracula Green
			url: "#ffb86c", // Dracula Orange
			file: "#ff5555", // Dracula Red
		};

		return (
			<div className="relative w-full h-full flex items-center justify-center bg-[#0a0e19]/50 rounded-2xl overflow-hidden border border-white/5 backdrop-blur-sm shadow-2xl">
				{/* Watermark/Logo */}
				<div className="absolute bottom-6 right-8 opacity-20 pointer-events-none select-none">
					<p className="text-sm font-black tracking-widest uppercase text-white/50">
						DeepMail{" "}
						<span className="text-primary">Intelligence</span>
					</p>
				</div>

				<ForceGraph2D
					ref={fgRef}
					graphData={graphData}
					width={width}
					height={height}
					backgroundColor="transparent"
					nodeRelSize={6}
					nodeLabel={() => ""}
					nodePointerAreaPaint={(node: any, color, ctx) => {
						ctx.fillStyle = color;
						ctx.beginPath();
						ctx.arc(
							node.x || 0,
							node.y || 0,
							10,
							0,
							2 * Math.PI,
							false,
						);
						ctx.fill();
					}}
					nodeCanvasObject={(nodeItem: any, ctx, globalScale) => {
						const node = nodeItem as GraphNode;
						ctx.shadowBlur = 0;
						ctx.shadowColor = "transparent";

						const color = nodeColors[node.type] || "#ffffff";
						const size = node.type === "email" ? 14 : 10;
						const radius = size / 2;

						// Draw malicious glow
						if (node.isMalicious || node.risk === "Critical") {
							ctx.shadowColor = "#ff5555";
							ctx.shadowBlur = 15;
							// Pulsing effect using global time if possible, but for now simple glow
							ctx.strokeStyle = "#ff5555";
							ctx.lineWidth = 2 / globalScale;
							ctx.beginPath();
							ctx.arc(
								node.x || 0,
								node.y || 0,
								radius + 2,
								0,
								2 * Math.PI,
								false,
							);
							ctx.stroke();
						}

						// Main node circle
						ctx.fillStyle = color;
						ctx.beginPath();
						ctx.arc(
							node.x || 0,
							node.y || 0,
							radius,
							0,
							2 * Math.PI,
							false,
						);
						ctx.fill();

						// Highlight border if selected or hovered
						if (hoveredNode?.id === node.id) {
							ctx.strokeStyle = "#ffffff";
							ctx.lineWidth = 2 / globalScale;
							ctx.beginPath();
							ctx.arc(
								node.x || 0,
								node.y || 0,
								radius + 1,
								0,
								2 * Math.PI,
								false,
							);
							ctx.stroke();
						}

						ctx.shadowBlur = 0; // reset

						// Labels
						if (
							showLabels &&
							(globalScale > 1.5 || node.type === "email")
						) {
							const label =
								node.value.length > 20
									? node.value.slice(0, 17) + "..."
									: node.value;
							const fontSize = 11 / globalScale;
							ctx.font = `${node.type === "email" ? "bold " : ""}${fontSize}px var(--font-body)`;
							ctx.textAlign = "center";
							ctx.textBaseline = "top";
							ctx.fillStyle = "rgba(255, 255, 255, 0.8)";
							ctx.fillText(
								label,
								node.x || 0,
								(node.y || 0) + radius + 4,
							);

							// Node type label (faint)
							ctx.font = `${8 / globalScale}px var(--font-body)`;
							ctx.fillStyle = "rgba(255, 255, 255, 0.4)";
							ctx.fillText(
								node.type.toUpperCase(),
								node.x || 0,
								(node.y || 0) + radius + 14,
							);
						}
					}}
					linkColor={() => "rgba(255, 255, 255, 0.15)"}
					linkWidth={(l) => (l.type === "attached_to" ? 2 : 1)}
					linkDirectionalParticles={2}
					linkDirectionalParticleSpeed={0.005}
					linkDirectionalParticleWidth={2}
					linkDirectionalParticleColor={(l: any) => {
						const targetNode =
							typeof l.target === "object"
								? l.target
								: graphData.nodes.find(
										(n) => n.id === l.target,
									);
						return targetNode?.isMalicious ? "#ff5555" : "#bd93f9";
					}}
					onNodeHover={(node: any) => {
						setHoveredNode(node as GraphNode);
						document.body.style.cursor = node
							? "pointer"
							: "default";
					}}
					onNodeClick={(nodeItem: any) => {
						onNodeClick?.(nodeItem as GraphNode);
					}}
					onNodeDragEnd={(node) => {
						if (node) {
							node.fx = node.x;
							node.fy = node.y;
						}
					}}
					onBackgroundClick={() => {
						onNodeClick?.(null as any);
					}}
				/>

				{/* Floating Tooltip */}
				<AnimatePresence>
					{hoveredNode && (
						<motion.div
							initial={{ opacity: 0, x: 20 }}
							animate={{ opacity: 1, x: 0 }}
							exit={{ opacity: 0, x: 20 }}
							className="absolute top-4 right-4 z-50 pointer-events-none"
						>
							<div className="bg-surface-container-highest/90 border border-outline-variant/30 rounded-xl px-4 py-3 shadow-2xl backdrop-blur-xl min-w-[200px]">
								<div className="flex items-center justify-between mb-2">
									<span
										className="text-[9px] font-bold uppercase tracking-wider px-2 py-1 rounded-sm border border-white/10"
										style={{
											color: nodeColors[hoveredNode.type],
										}}
									>
										{hoveredNode.type}
									</span>
									<span
										className={`text-[9px] font-bold uppercase tracking-wider ${
											hoveredNode.risk === "Critical"
												? "text-dracula-red"
												: "text-dracula-green"
										}`}
									>
										{hoveredNode.risk}
									</span>
								</div>
								<p className="text-sm font-bold text-on-surface mb-1 font-mono break-all line-clamp-2">
									{hoveredNode.value}
								</p>
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		);
	},
);

export default ThreatGraph;

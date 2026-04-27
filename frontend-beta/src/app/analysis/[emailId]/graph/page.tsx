"use client";

import { useEffect, useState, useRef } from "react";
import dynamic from "next/dynamic";
import { useParams } from "next/navigation";
import Link from "next/link";
import { getResults } from "@/lib/api";
import { useThreatGraph } from "@/hooks/useThreatGraph";
import type { ThreatGraphHandle } from "@/components/dashboard/ThreatGraph";
import { AnimatePresence } from "motion/react";
import GraphSidebar from "@/components/graph/GraphSidebar";

const ThreatGraph = dynamic(
	() => import("@/components/dashboard/ThreatGraph"),
	{
		ssr: false,
	},
);

export default function EmailGraphPage() {
	const params = useParams();
	const emailId = params.emailId as string;
	const graphRef = useRef<ThreatGraphHandle>(null);

	const {
		graphData,
		addEmailAnalysis,
		selectedNode,
		setSelectedNodeId,
		toggleNodeCollapse,
	} = useThreatGraph();

	const [isLoading, setIsLoading] = useState(true);
	const [dimensions, setDimensions] = useState({ width: 0, height: 0 });

	useEffect(() => {
		const updateDimensions = () => {
			setDimensions({
				width: window.innerWidth,
				height: window.innerHeight,
			});
		};
		updateDimensions();
		window.addEventListener("resize", updateDimensions);
		return () => window.removeEventListener("resize", updateDimensions);
	}, []);

	useEffect(() => {
		async function loadData() {
			try {
				const report = await getResults(emailId);
				addEmailAnalysis(report);
			} catch (error) {
				console.error("Failed to load email graph data", error);
			} finally {
				setIsLoading(false);
			}
		}
		loadData();
	}, [emailId, addEmailAnalysis]);

	return (
		<main className="relative flex-1 bg-[#06080f] overflow-hidden flex flex-col font-body">
			{/* Breadcrumb Header */}
			<div className="absolute top-6 left-8 z-30 flex items-center gap-2 text-xs text-white/40">
				<Link href="/" className="hover:text-primary transition-colors">
					Dashboard
				</Link>
				<span className="material-symbols-outlined text-xs">
					chevron_right
				</span>
				<Link
					href={`/analysis/${emailId}`}
					className="hover:text-primary transition-colors"
				>
					Analysis
				</Link>
				<span className="material-symbols-outlined text-xs">
					chevron_right
				</span>
				<span className="text-white font-bold">Graph Perspective</span>
			</div>

			<div className="flex-1 relative">
				{isLoading ? (
					<div className="absolute inset-0 flex items-center justify-center flex-col gap-4">
						<div className="w-12 h-12 border-4 border-primary/20 border-t-primary rounded-full animate-spin" />
						<p className="text-[10px] font-black tracking-[0.4em] uppercase text-white/30 animate-pulse">
							Mapping IOC Relations...
						</p>
					</div>
				) : (
					<ThreatGraph
						ref={graphRef}
						graphData={graphData}
						width={dimensions.width - 256}
						height={dimensions.height - 64}
						onNodeClick={(node) =>
							setSelectedNodeId(node?.id || null)
						}
					/>
				)}

				{/* Floating Controls */}
				<div className="absolute right-10 top-10 z-60">
					<div className="bg-[#12141c]/95 border border-white/20 p-2 rounded-2xl shadow-2xl flex flex-col gap-2 backdrop-blur-3xl">
						<button
							onClick={() => graphRef.current?.zoomIn()}
							className="w-10 h-10 rounded-xl flex items-center justify-center text-white hover:bg-primary hover:text-on-primary transition-all bg-white/5 border border-white/10 group"
							title="Zoom In"
						>
							<span className="material-symbols-outlined text-lg leading-none group-hover:scale-110 transition-transform">
								add
							</span>
						</button>
						<button
							onClick={() => graphRef.current?.zoomOut()}
							className="w-10 h-10 rounded-xl flex items-center justify-center text-white hover:bg-primary hover:text-on-primary transition-all bg-white/5 border border-white/10 group"
							title="Zoom Out"
						>
							<span className="material-symbols-outlined text-lg leading-none group-hover:scale-90 transition-transform">
								remove
							</span>
						</button>
						<div className="h-px bg-white/10 mx-1 my-1" />
						<button
							onClick={() => graphRef.current?.center()}
							className="w-10 h-10 rounded-xl bg-primary text-on-primary flex items-center justify-center shadow-lg transform hover:scale-105 active:scale-95 transition-all border border-white/20 group"
							title="Center View"
						>
							<span className="material-symbols-outlined text-lg group-hover:rotate-12 transition-transform">
								my_location
							</span>
						</button>
					</div>
				</div>

				{/* Legend */}
				<div className="absolute right-8 bottom-8 z-20 flex gap-6 px-6 py-4 bg-black/40 backdrop-blur-md rounded-2xl border border-white/5 pointer-events-none">
					{[
						{ label: "Email", color: "#bd93f9" },
						{ label: "IP", color: "#8be9fd" },
						{ label: "Domain", color: "#50fa7b" },
						{ label: "URL", color: "#ffb86c" },
						{ label: "File", color: "#ff5555" },
					].map((item) => (
						<div
							key={item.label}
							className="flex items-center gap-2"
						>
							<div
								className="w-2 h-2 rounded-full"
								style={{ backgroundColor: item.color }}
							/>
							<span className="text-[9px] font-black tracking-widest text-white/40 uppercase">
								{item.label}
							</span>
						</div>
					))}
				</div>

				{/* Selection Sidebar */}
				<AnimatePresence>
					{selectedNode && (
						<GraphSidebar
							node={selectedNode}
							onClose={() => setSelectedNodeId(null)}
							onExpand={(id) => toggleNodeCollapse(id)}
						/>
					)}
				</AnimatePresence>
			</div>
		</main>
	);
}

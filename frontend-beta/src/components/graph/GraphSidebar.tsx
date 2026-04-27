"use client";

import { GraphNode } from "@/hooks/useThreatGraph";
import { motion, AnimatePresence } from "motion/react";
import { formatUtcTime } from "@/lib/format";

interface GraphSidebarProps {
	node: GraphNode | null;
	onClose: () => void;
	onExpand: (id: string) => void;
}

export default function GraphSidebar({
	node,
	onClose,
	onExpand,
}: GraphSidebarProps) {
	if (!node) return null;

	const nodeColors: Record<string, string> = {
		email: "#bd93f9",
		ip: "#8be9fd",
		domain: "#50fa7b",
		url: "#ffb86c",
		file: "#ff5555",
	};

	return (
		<motion.div
			initial={{ x: 300, opacity: 0 }}
			animate={{ x: 0, opacity: 1 }}
			exit={{ x: 300, opacity: 0 }}
			className="absolute right-6 top-6 bottom-6 w-[320px] bg-surface-container-high/95 backdrop-blur-3xl border border-white/10 rounded-3xl shadow-2xl z-50 flex flex-col overflow-hidden"
		>
			{/* Header */}
			<div className="p-6 border-b border-white/5 flex items-center justify-between">
				<div className="flex items-center gap-3">
					<div
						className="w-10 h-10 rounded-2xl flex items-center justify-center shadow-lg transform rotate-3"
						style={{
							backgroundColor: nodeColors[node.type] + "20",
						}}
					>
						<span
							className="material-symbols-outlined text-2xl"
							style={{ color: nodeColors[node.type] }}
						>
							{node.type === "email"
								? "drafts"
								: node.type === "ip"
									? "public"
									: node.type === "domain"
										? "language"
										: node.type === "url"
											? "link"
											: "description"}
						</span>
					</div>
					<div>
						<h3 className="text-sm font-black tracking-widest uppercase text-white/50">
							Details
						</h3>
						<p
							className="text-xs font-bold"
							style={{ color: nodeColors[node.type] }}
						>
							{node.type.toUpperCase()}
						</p>
					</div>
				</div>
				<button
					onClick={onClose}
					className="w-8 h-8 rounded-full flex items-center justify-center text-white/40 hover:text-white hover:bg-white/5 transition-colors"
				>
					<span className="material-symbols-outlined text-sm">
						close
					</span>
				</button>
			</div>

			{/* Content */}
			<div className="flex-1 p-6 space-y-8 overflow-y-auto no-scrollbar">
				{/* Primary Value */}
				<section>
					<label className="text-[10px] font-black tracking-widest text-white/30 uppercase block mb-3">
						Target Value
					</label>
					<div className="bg-black/20 p-4 rounded-2xl border border-white/5 relative group overflow-hidden">
						<p className="text-sm font-bold text-white break-all font-mono leading-relaxed relative z-10">
							{node.value}
						</p>
						<div
							className="absolute top-0 right-0 w-32 h-32 blur-3xl opacity-10 group-hover:opacity-20 transition-opacity"
							style={{ backgroundColor: nodeColors[node.type] }}
						/>
					</div>
				</section>

				{/* Risk Status */}
				<section>
					<label className="text-[10px] font-black tracking-widest text-white/30 uppercase block mb-3">
						Risk Intelligence
					</label>
					<div className="flex items-center gap-3">
						<div
							className={`px-4 py-2 rounded-xl text-xs font-black tracking-widest uppercase flex items-center gap-2 border ${
								node.risk === "Critical"
									? "bg-dracula-red/10 text-dracula-red border-dracula-red/20"
									: "bg-dracula-green/10 text-dracula-green border-dracula-green/20"
							}`}
						>
							<span
								className={`w-2 h-2 rounded-full ${
									node.risk === "Critical"
										? "bg-dracula-red animate-pulse"
										: "bg-dracula-green"
								}`}
							/>
							{node.risk}
						</div>
						{node.isMalicious && (
							<div className="px-4 py-2 rounded-xl bg-dracula-red text-white text-[10px] font-black tracking-widest uppercase">
								Threat
							</div>
						)}
					</div>
				</section>

				{/* Metadata Table */}
				<section>
					<label className="text-[10px] font-black tracking-widest text-white/30 uppercase block mb-3">
						Timeline & Origin
					</label>
					<div className="space-y-4">
						<div className="flex justify-between items-center py-2 border-b border-white/5">
							<span className="text-[11px] text-white/40">
								First Observed
							</span>
							<span className="text-[11px] text-white font-medium">
								{node.metadata?.first_seen
									? formatUtcTime(node.metadata.first_seen)
									: "2026-03-31 15:44"}
							</span>
						</div>
						<div className="flex justify-between items-center py-2 border-b border-white/5">
							<span className="text-[11px] text-white/40">
								Last Seen
							</span>
							<span className="text-[11px] text-white font-medium">
								{node.metadata?.last_seen
									? formatUtcTime(node.metadata.last_seen)
									: "Just now"}
							</span>
						</div>
						{node.metadata?.country && (
							<div className="flex justify-between items-center py-2 border-b border-white/5">
								<span className="text-[11px] text-white/40">
									Geography
								</span>
								<span className="text-[11px] text-white font-medium">
									{node.metadata.country}
								</span>
							</div>
						)}
					</div>
				</section>
			</div>

			{/* Actions */}
			<div className="p-6 border-t border-white/5 space-y-3">
				<button
					onClick={() => onExpand(node.id)}
					className="w-full py-4 rounded-2xl bg-primary text-on-primary text-xs font-black tracking-widest uppercase hover:brightness-110 transition-all flex items-center justify-center gap-2 shadow-xl shadow-primary/20"
				>
					<span className="material-symbols-outlined text-sm">
						hub
					</span>
					Expand Connections
				</button>
				<button className="w-full py-4 rounded-2xl bg-white/5 text-white/80 text-xs font-black tracking-widest uppercase hover:bg-white/10 transition-all flex items-center justify-center gap-2">
					<span className="material-symbols-outlined text-sm">
						visibility
					</span>
					View Sources
				</button>
			</div>
		</motion.div>
	);
}

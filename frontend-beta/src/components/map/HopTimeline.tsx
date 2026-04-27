"use client";

import type { HopTimelinePoint } from "@/lib/types";

interface HopTimelineProps {
	hops: HopTimelinePoint[];
	activeHop: number;
	onChange: (index: number) => void;
}

export default function HopTimeline({ hops, activeHop, onChange }: HopTimelineProps) {
	if (hops.length === 0) {
		return null;
	}

	return (
		<div className="glass-panel rounded-xl p-4 md:p-5 space-y-3">
			<div className="flex items-center justify-between gap-3">
				<h3 className="text-xs font-bold uppercase tracking-widest text-on-surface">Received Hop Playback</h3>
				<span className="text-[10px] text-outline">
					Hop {Math.min(activeHop + 1, hops.length)}/{hops.length}
				</span>
			</div>

			<input
				type="range"
				min={0}
				max={Math.max(hops.length - 1, 0)}
				value={Math.min(activeHop, hops.length - 1)}
				onChange={(event) => onChange(Number(event.target.value))}
				className="w-full accent-cyan-300"
			/>

			<div className="grid gap-2 max-h-36 overflow-y-auto pr-1">
				{hops.map((hop, index) => (
					<button
						key={`${hop.hop}-${hop.ip ?? "na"}`}
						onClick={() => onChange(index)}
						className={`text-left p-2.5 rounded-lg border transition-colors ${
							index === activeHop
								? "border-cyan-300/50 bg-cyan-300/10"
								: "border-outline-variant/20 bg-surface-container-low/30 hover:bg-surface-container-low/50"
						}`}
					>
						<p className="text-[11px] font-semibold text-on-surface">
							Hop {hop.hop} {hop.ip ? `• ${hop.ip}` : "• (no IP)"}
						</p>
						<p className="text-[10px] text-on-surface-variant mt-0.5 truncate">
							{hop.from_host ?? "unknown source"} → {hop.by_host ?? "unknown relay"}
						</p>
					</button>
				))}
			</div>
		</div>
	);
}

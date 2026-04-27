"use client";

import Link from "next/link";
import { motion, AnimatePresence } from "framer-motion";
import type { GeoMapPoint } from "@/lib/types";

interface IpSidebarProps {
	emailId: string;
	selected: GeoMapPoint | null;
	totalPoints: number;
	unresolvedCount: number;
}

const riskMeta: Record<
	GeoMapPoint["risk"],
	{ label: string; dot: string; text: string }
> = {
	critical: { label: "Critical", dot: "#ff5555", text: "text-error" },
	high: { label: "High", dot: "#bd93f9", text: "text-primary" },
	medium: { label: "Medium", dot: "#ffb86c", text: "text-dracula-orange" },
	low: { label: "Low", dot: "#50fa7b", text: "text-tertiary" },
};

export default function IpSidebar({
	emailId,
	selected,
	totalPoints,
	unresolvedCount,
}: IpSidebarProps) {
	return (
		<div className="glass-panel rounded-xl h-full flex flex-col overflow-hidden">
			<div className="px-6 py-5 border-b border-outline-variant/10">
				<h4 className="text-sm font-bold tracking-widest uppercase font-headline">
					IP Intelligence
				</h4>
				<p className="text-[11px] text-on-surface-variant mt-1">
					{totalPoints} resolved location{totalPoints === 1 ? "" : "s"}
					{unresolvedCount > 0 ? ` • ${unresolvedCount} unavailable` : ""}
				</p>
			</div>

			<div className="flex-1 p-5">
				<AnimatePresence mode="wait">
					{selected ? (
						<motion.div
							key={selected.id}
							initial={{ opacity: 0, y: 8 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0, y: -8 }}
							transition={{ duration: 0.2 }}
							className="space-y-4"
						>
							<div className="flex items-center justify-between">
								<span className="text-[10px] text-outline uppercase tracking-widest font-bold">
									Selected Node
								</span>
								<span className={`text-xs font-bold ${riskMeta[selected.risk].text}`}>
									{riskMeta[selected.risk].label}
								</span>
							</div>

							<div className="p-4 rounded-lg bg-surface-container-low/50 border border-outline-variant/10 space-y-3">
								<div>
									<p className="text-[10px] text-outline uppercase tracking-widest">
										IP Address
									</p>
									<p className="text-sm font-mono text-on-surface break-all">
										{selected.ip}
									</p>
								</div>

								<div className="grid grid-cols-2 gap-3 text-xs">
									<div>
										<p className="text-outline">Country</p>
										<p className="text-on-surface font-semibold">
											{selected.country}
										</p>
									</div>
									<div>
										<p className="text-outline">City</p>
										<p className="text-on-surface font-semibold">
											{selected.city || "Unknown"}
										</p>
									</div>
									<div>
										<p className="text-outline">Coordinates</p>
										<p className="text-on-surface font-semibold">
											{selected.lat.toFixed(4)}, {selected.lon.toFixed(4)}
										</p>
									</div>
									<div>
										<p className="text-outline">Org</p>
										<p className="text-on-surface font-semibold truncate">
											{selected.org || "Unknown"}
										</p>
									</div>
									<div>
										<p className="text-outline">ASN</p>
										<p className="text-on-surface font-semibold">
											{selected.asn ?? "Unknown"}
										</p>
									</div>
									<div>
										<p className="text-outline">Proxy/TOR</p>
										<p className="text-on-surface font-semibold">
											{selected.is_proxy ? "Proxy" : "No Proxy"}
											{selected.is_tor ? " • TOR" : ""}
										</p>
									</div>
									<div>
										<p className="text-outline">Abuse score</p>
										<p className="text-on-surface font-semibold">
											{selected.abuse_confidence ?? "n/a"}
										</p>
									</div>
								</div>

								<div className="flex items-center justify-between pt-1">
									<div className="flex items-center gap-2">
										<span
											className="w-2.5 h-2.5 rounded-full"
											style={{ backgroundColor: riskMeta[selected.risk].dot }}
										/>
										<span className="text-[11px] text-on-surface-variant">
											Confidence: {(selected.confidence_score * 100).toFixed(0)}%
										</span>
									</div>
								</div>
							</div>

							<Link
								href={`/analysis/${emailId}`}
								className="w-full px-4 py-3 rounded-lg bg-primary text-on-primary text-xs font-bold uppercase tracking-widest flex items-center justify-center gap-2 hover:opacity-90 transition-opacity"
							>
								<span className="material-symbols-outlined text-sm">
									description
								</span>
								View Full Report
							</Link>
						</motion.div>
					) : (
						<motion.div
							key="empty"
							initial={{ opacity: 0, y: 8 }}
							animate={{ opacity: 1, y: 0 }}
							exit={{ opacity: 0 }}
							className="h-full flex flex-col items-center justify-center text-center px-4"
						>
							<span className="material-symbols-outlined text-3xl text-outline/60">
								location_on
							</span>
							<p className="mt-3 text-sm text-on-surface">Select an IP node</p>
							<p className="text-xs text-on-surface-variant mt-1">
								Click any map marker to inspect geolocation details.
							</p>
						</motion.div>
					)}
				</AnimatePresence>
			</div>
		</div>
	);
}
